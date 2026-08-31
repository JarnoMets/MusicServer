#!/bin/bash
AUTH_TOKEN="${AUTH_TOKEN:?Set AUTH_TOKEN env var}"
BASE_URL="https://music.jarnomets.com/api"

# Fetch all tracks where title starts with artist + " - "
echo "Fetching tracks..."
curl -s "$BASE_URL/music?limit=2000" > all_tracks.json

# Process tracks
jq -r '.[] | (.artist|tostring) as $a | select(.title | startswith($a + " - ")) | .id + "|" + $a + "|" + .title' all_tracks.json | while IFS='|' read -r id artist title; do
    # Remove artist prefix: "Artist - Title" -> "Title"
    # We remove exactly "$artist - " from the start
    new_title="${title#$artist - }"
    
    # Clean suffixes
    # Using sed to handle multiple common suffixes at once
    cleaned_title=$(echo "$new_title" | sed -E '
        s/\((Official Video|Official Music Video|Official Audio|Official Visualiser|Audio|Music Video|Lyric Video|HD|HQ|OUT NOW)\)//gI;
        s/\[(Official Video|Official Music Video|Official Audio|Official Visualiser|Audio|Music Video|Lyric Video|HD|HQ|OUT NOW)\]//gI;
        s/\s+$//g
    ')
    
    if [ "$title" != "$cleaned_title" ]; then
        echo "Updating ID $id:"
        echo "  Old info : $artist | $title"
        echo "  New title: $cleaned_title"
        
        # URL encode only the ID for the path
        # encoded_id is already just a UUID so no need for encoding
        
        json_payload=$(jq -n --arg title "$cleaned_title" '{"title": $title}')
        response=$(curl -s -X PATCH "$BASE_URL/music/$id" \
             -H "Authorization: Bearer $AUTH_TOKEN" \
             -H "Content-Type: application/json" \
             -d "$json_payload")
        
        # Check if response is empty or unauthorized
        if [[ "$response" == *"unauthorized"* ]]; then
             echo "Error: Unauthorized. Check token."
             exit 1
        fi
        echo "  Response : $response"
        echo "---"
    fi
done
