#!/bin/bash
AUTH_TOKEN="${AUTH_TOKEN:?Set AUTH_TOKEN env var}"
BASE_URL="https://music.jarnomets.com/api"

echo "Fixing broken closing brackets..."
curl -s "$BASE_URL/music?limit=2000" > all_tracks.json

jq -r '.[] | .id + "|" + .title' all_tracks.json | while IFS='|' read -r id title; do
    new_title="$title"
    
    # Check for unclosed (
    open_p=$(echo "$title" | grep -o "(" | wc -l)
    close_p=$(echo "$title" | grep -o ")" | wc -l)
    if [ "$open_p" -gt "$close_p" ]; then
        new_title="$new_title)"
    fi
    
    # Check for unclosed [
    open_s=$(echo "$title" | grep -o "\[" | wc -l)
    close_s=$(echo "$title" | grep -o "\]" | wc -l)
    if [ "$open_s" -gt "$close_s" ]; then
        new_title="$new_title]"
    fi
    
    if [ "$title" != "$new_title" ]; then
        echo "Fixing ID $id: $title -> $new_title"
        curl -s -X PATCH "$BASE_URL/music/$id" \
             -H "Authorization: Bearer $AUTH_TOKEN" \
             -H "Content-Type: application/json" \
             -d "{\"title\": \"$new_title\"}" > /dev/null
    fi
done
