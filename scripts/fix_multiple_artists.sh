#!/bin/bash
AUTH_TOKEN="${AUTH_TOKEN:?Set AUTH_TOKEN env var}"
BASE_URL="https://music.jarnomets.com/api"

echo "Fetching all tracks for multiple artist cleanup..."
curl -s "$BASE_URL/music?limit=2000" > all_tracks.json

# Process tracks
jq -r '.[] | select(.artist != null and .title != null) | select(.title | contains(" - ")) | .id + "|" + .artist + "|" + .title' all_tracks.json | while IFS='|' read -r id artist title; do
    # Extract the part before the first " - "
    prefix="${title%% - *}"
    # Extract the part after the first " - "
    suffix="${title#* - }"

    # If they are the same, it means " - " was not found (but we select only those that contain it)
    if [ "$prefix" == "$title" ]; then
        continue
    fi

    # Check if the current artist is part of the prefix but not exactly the prefix
    # Also handle case-insensitivity just in case
    low_artist=$(echo "$artist" | tr '[:upper:]' '[:lower:]')
    low_prefix=$(echo "$prefix" | tr '[:upper:]' '[:lower:]')

    if [[ "$low_prefix" == *"$low_artist"* ]] && [[ "$low_prefix" != "$low_artist" ]]; then
        # Check for separators in prefix: comma, ampersand, ' x ', ' and ', ' feat. ', ' ft. ', ' & '
        if [[ "$prefix" =~ "," ]] || [[ "$prefix" =~ " & " ]] || [[ "$prefix" =~ " x " ]] || [[ "$prefix" =~ " and " ]] || [[ "$prefix" =~ " feat. " ]] || [[ "$prefix" =~ " ft. " ]]; then
            
            # Clean "Original Mix" from suffix while we are at it
            cleaned_suffix=$(echo "$suffix" | sed -E 's/[ \t]*[\(\[]Original Mix[\)\]]//gI; s/[ \t]+Original Mix//gI; s/  +/ /g; s/^ //g; s/ $//g')

            echo "Updating ID $id:"
            echo "  Old Artist: $artist"
            echo "  Old Title:  $title"
            echo "  New Artist: $prefix"
            echo "  New Title:  $cleaned_suffix"

            json_payload=$(jq -n --arg artist "$prefix" --arg title "$cleaned_suffix" '{"artist": $artist, "title": $title}')
            curl -s -X PATCH "$BASE_URL/music/$id" \
                 -H "Authorization: Bearer $AUTH_TOKEN" \
                 -H "Content-Type: application/json" \
                 -d "$json_payload" > /dev/null
        fi
    fi
done
