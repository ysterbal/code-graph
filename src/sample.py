import os
from typing import List, Optional


class CodeParser:
    """
    A class to represent a parser that processes source code for GraphRAG.
    """

    def __init__(self, language: str):
        self.language = language
        self.nodes = []

    def add_node(self, name: str, node_type: str) -> None:
        """Adds a node to the internal graph representation."""
        node = {"name": name, "type": node_type}
        self.nodes.append(node)
        print(f"Added {node_type}: {name}")


def process_directory(path: str) -> List[str]:
    """
    Scans a directory for python files.
    """
    files = [f for f in os.listdir(path) if f.endswith(".py")]
    return files


def main():
    # Initialize the parser
    parser = CodeParser(language="Python")

    # Define a sample path
    current_dir = "./src"

    # Process files
    try:
        python_files = process_directory(current_dir)
        for file_name in python_files:
            parser.add_node(file_name, "File")
    except FileNotFoundError:
        print("Directory not found.")


if __name__ == "__main__":
    main()
