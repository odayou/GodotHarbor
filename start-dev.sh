#!/bin/bash
echo "Starting Godot Harbor Development Server..."
echo ""
echo "Installing dependencies..."
npm install
echo ""
echo "Starting Tauri development mode..."
npm run tauri dev
