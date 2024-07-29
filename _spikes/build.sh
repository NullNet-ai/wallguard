#!/bin/bash

# Check if a .c file is provided as an argument
if [ -z "$1" ]; then
  echo "Usage: $0 <filename.c>"
  exit 1
fi

# Extract the base name of the .c file without the extension
base_name=$(basename "$1" .c)

# Compile the .c file
gcc -o "$base_name" "$1"

# Check if compilation was successful
if [ $? -eq 0 ]; then
  # Execute the compiled program
  ./"$base_name"
  
  # Remove the executable
  rm -f "$base_name"
else
  echo "Compilation failed."
  exit 1
fi