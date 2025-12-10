#!/bin/bash

# Test script for YouTube Downloader API

BASE_URL="http://localhost:8081/api"

echo "Testing YouTube Downloader API..."

# Test health check first
echo "1. Testing health check..."
curl -s "$BASE_URL/health" | jq '.'

echo -e "\n2. Testing YouTube download start..."

# Test download start (replace with a real YouTube URL for testing)
RESPONSE=$(curl -s -X POST "$BASE_URL/youtube/download" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
    "output_dir": "/tmp/music_test",
    "limit": 1,
    "max_concurrent": 1,
    "audio_quality": "192"
  }')

echo "$RESPONSE" | jq '.'

# Extract session ID
SESSION_ID=$(echo "$RESPONSE" | jq -r '.session_id')

if [ "$SESSION_ID" != "null" ] && [ "$SESSION_ID" != "" ]; then
  echo -e "\n3. Testing progress check..."
  sleep 2
  curl -s "$BASE_URL/youtube/progress/$SESSION_ID" | jq '.'
  
  echo -e "\n4. Testing cancel..."
  curl -s -X POST "$BASE_URL/youtube/cancel/$SESSION_ID" | jq '.'
else
  echo "Failed to get session ID from download start"
fi

echo -e "\nTest completed!"