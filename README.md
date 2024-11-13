# Spriterator

`Spriterator` is a Rust library that generates a sprite sheet from images in a specified directory. It scans directories for images, arranges them into a grid layout, and saves the resulting sprite sheet to a specified output path.

This library supports common image formats like PNG, JPEG, GIF, and WebP, and uses parallel processing for efficient image loading.

## Features

- **Recursive Directory Scanning**: Finds all images within nested directories.
- **Parallel Processing**: Speeds up loading and processing of images.
- **Flexible Output Size**: Allows specifying a maximum width for the generated sprite sheet.
- **Supported Formats**: Handles common image formats (`png`, `jpg`, `jpeg`, `gif`, `bmp`, `ico`, `tiff`, `webp`).
