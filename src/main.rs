use code_graph_tool::{config, db, graph, parser};

mod llm;

use clap::Parser;
use graph::CodeGraph;
use tracing::{debug, info, warn, Level};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to a file or directory to parse
    #[arg(short, long, default_value = "src")]
    path: String,

    /// The target node to fetch context for (e.g., "src/sample.py:main")
    #[arg(short, long)]
    target: Option<String>,

    /// Function or class name to discover (requires --discover-name)
    #[arg(long = "discover-name")]
    discover_name: Option<String>,

    /// The prompt to send to the LLM (optional when using --discover-name)
    #[arg(long, default_value = "")]
    prompt: String,

    /// Only search for nodes, don't call LLM
    #[arg(long)]
    search_only: bool,

    /// Maximum depth for dependency context retrieval
    #[arg(short, long, default_value_t = 2)]
    depth: u32,

    /// Maximum token limit for context generation (approximate)
    #[arg(short = 'm', long, default_value_t = 2000)]
    max_tokens: usize,

    /// Path to configuration file (optional)
    #[arg(long = "config")]
    config_file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("code_graph_tool=info".parse()?),
        )
        .init();

    let args = Args::parse();

    info!("Initializing GraphRAG tool");

    // Load configuration from file or environment
    let config = if let Some(ref config_path) = args.config_file {
        info!("Loading configuration from file: {}", config_path);
        config::Config::from_file(config_path)?
    } else {
        info!("Using default configuration (environment variables)");
        config::Config::from_env()
    };
    info!(
        "Configuration loaded: max_depth={}, max_tokens={}",
        config.parser.max_depth, config.parser.max_tokens
    );

    // Setup SQLite DB connection pool
    let db_url = config.db_url();
    info!("Database URL: {}", db_url);
    let pool = db::init_db(&db_url).await?;

    let mut graph = CodeGraph::new();

    // Parse the specified file or directory
    let target_path = std::path::Path::new(&args.path);
    info!("Target path: {:?}", target_path);

    // Clean up files that have been deleted from disk
    debug!("Checking for deleted files...");
    let db_files = db::get_all_files(&pool).await?;
    for file_path in db_files {
        let path = std::path::Path::new(&file_path);
        // Only prune if the DB file belongs to the scanned path but no longer exists
        if path.starts_with(target_path) && !path.exists() {
            warn!("Pruning deleted file: {}", file_path);
            db::clear_file_state(&pool, &file_path).await?;
        }
    }

    if target_path.is_file() {
        if let Some(p) = target_path.to_str() {
            let code = std::fs::read_to_string(p)?;
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&code, &mut hasher);
            let checksum = std::hash::Hasher::finish(&hasher).to_string();

            if db::get_file_checksum(&pool, p).await?.as_deref() == Some(&checksum) {
                println!("Skipping {} (unchanged)", p);
            } else {
                println!("Parsing {}...", p);
                db::clear_file_state(&pool, p).await?;
                parser::parse_file(p, &code, &checksum, &mut graph)?;
            }
        } else {
            eprintln!("Invalid file path provided.");
        }
    } else if target_path.is_dir() {
        for entry in walkdir::WalkDir::new(target_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let valid_exts = ["py", "rs", "js", "ts", "go", "cpp", "cc", "cxx", "c", "h"];
            if path.is_file()
                && path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|ext| valid_exts.contains(&ext))
            {
                if let Some(p) = path.to_str() {
                    let code = match std::fs::read_to_string(p) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("Failed to read {}: {}", p, e);
                            continue;
                        }
                    };
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    std::hash::Hash::hash(&code, &mut hasher);
                    let checksum = std::hash::Hasher::finish(&hasher).to_string();

                    if db::get_file_checksum(&pool, p).await?.as_deref() == Some(&checksum) {
                        debug!("Skipping {} (unchanged)", p);
                    } else {
                        info!("Parsing {}...", p);
                        db::clear_file_state(&pool, p).await?;
                        if let Err(e) = parser::parse_file(p, &code, &checksum, &mut graph) {
                            eprintln!("Failed to parse {}: {}", p, e);
                        }
                    }
                }
            }
        }
    }
    let node_count = graph.nodes.node_count();
    info!("Successfully built a graph with {} nodes", node_count);

    // Save graph to DB
    db::save_graph(&graph, &pool).await?;
    info!("Graph persisted to SQLite database");

    // Phase 2b: Discovery mode - find functions/classes by name
    if let Some(discover_name) = &args.discover_name {
        info!("Searching for nodes matching pattern: {}", discover_name);
        let matches = db::search_nodes_by_name(&pool, discover_name).await?;

        if matches.is_empty() {
            eprintln!("No nodes found matching '{}'", discover_name);
            std::process::exit(1);
        }

        info!("Found {} matching node(s)", matches.len());
        for (id, node_type) in &matches {
            debug!("  - {} ({})", id, node_type);
        }

        // If search-only mode, just display results and exit
        if args.search_only {
            return Ok(());
        }

        // If target not specified, use first match
        let target_node = args.target.as_ref().unwrap_or(&matches[0].0);
        info!("Using target: {}", target_node);
        debug!("Fetching dependency context for: {}", target_node);

        let context_items = db::get_dependency_context(
            &pool,
            target_node,
            config.parser.max_depth,
            config.parser.max_tokens,
        )
        .await?;
        let context_string = context_items.join("\n\n");

        let user_prompt = if args.prompt.is_empty() {
            format!("Analyze the following code node: {}", target_node)
        } else {
            args.prompt.clone()
        };

        debug!("Sending prompt to LLM...");
        let llm_response = llm::send_to_llm(user_prompt, context_string).await?;
        info!("LLM response received");
        println!("\n=== LLM Response ===\n{}", llm_response);
        return Ok(());
    }

    // Phase 3: Fetch context and send to LLM
    let target_node = args
        .target
        .as_ref()
        .ok_or("--target is required unless using --discover-name")?;
    info!("Fetching dependency context for: {}", target_node);

    let context_items = db::get_dependency_context(
        &pool,
        target_node,
        config.parser.max_depth,
        config.parser.max_tokens,
    )
    .await?;
    let context_string = context_items.join("\n\n");

    let user_prompt = args.prompt;
    debug!("Sending prompt to LLM...");

    let llm_response = llm::send_to_llm(user_prompt, context_string).await?;
    info!("LLM response received");
    println!("\n=== LLM Response ===\n{}", llm_response);

    Ok(())
}
