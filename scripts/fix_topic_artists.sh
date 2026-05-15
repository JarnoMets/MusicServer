#!/bin/bash
AUTH_TOKEN="${AUTH_TOKEN:?Set AUTH_TOKEN env var}"
BASE_URL="https://music.jarnomets.com/api"

# Get artists ending with - Topic except Release - Topic
topic_artists=$(curl -s "$BASE_URL/artists" | jq -r '.[] | .name' | grep " - Topic$" | grep -v "^Release - Topic$")

echo "Found the following artists to fix:"
echo "$topic_artists"
echo "---"

while IFS= read -r artist; do
    if [ -z "$artist" ]; then continue; fi
    
    new_artist="${artist% - Topic}"
    encoded_artist=$(echo -n "$artist" | jq -sRr @uri)
    
    echo "Renaming '$artist' -> '$new_artist'..."
    response=$(curl -s -X PUT "$BASE_URL/admin/artists/$encoded_artist/rename" \
         -H "Authorization: Bearer $AUTH_TOKEN" \
         -H "Content-Type: application/json" \
         -d "{\"new_name\": \"$new_artist\"}")
    
    echo "Response: $response"
done <<< "$topic_artists"
