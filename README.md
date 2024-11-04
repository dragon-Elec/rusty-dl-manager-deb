# Rusty Download Manager

![Rusty Download Manager Logo]

Rusty Download Manager is an asynchronous and performant download manager written in Rust. It is designed to provide a feature-rich experience, enabling users to efficiently manage their downloads with ease. Whether you're downloading large files or managing multiple downloads simultaneously, Rusty Download Manager has you covered.

## Features

- **Asynchronous Performance**: Built on Rust's powerful async capabilities, ensuring fast and efficient downloading without blocking the user interface.
  
- **Bandwidth Control**: Control your download speed and manage your bandwidth usage effectively.

- **Action on Save**: Customize what happens after a file is downloaded, from moving files to specific folders to executing scripts.

- **Browser Download Grabbing**: Seamlessly integrate with your web browser to grab and manage downloads directly from your browsing experience.

- **File Filtering**: Set filters to manage and categorize your downloads based on file types, sizes, and other criteria.

- **Total Bandwidth Plotting**: Visualize your download speeds and bandwidth usage over time with built-in plotting tools.

- **Auto-Retrying**: Automatically retry failed downloads, ensuring that your files are downloaded without manual intervention.

- **Cross-Compilable**: Easily compile and run on multiple platforms, making it accessible on various operating systems.

## Installation

To get started with Rusty Download Manager, clone the repository and build the project:

```bash
git clone https://github.com/yourusername/rusty-download-manager.git
cd rusty-download-manager
cargo build --release
