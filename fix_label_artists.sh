#!/bin/bash
AUTH_TOKEN="REDACTED_AUTH_TOKEN"
BASE_URL="https://music.jarnomets.com/api"

# Case-insensitive matching
shopt -s nocasematch

# Case insensitive matching
shopt -s nocasematch

# Labels or Channels to clean (regex match)
# Only split title if artist matches one of these
LABEL_REGEX="Music|Records|Record|Recordings|TV|Channel|Productions|Production|Label|Promo|Network|Sounds|Sound|Vevo|Official|Topic|Collective|Beat|Daily|Group|Uploader|Upload|Exclusive|Studio|Digital|Network|VEVO|DailyMusic|YouTube|Monstercat"

EXCEPTIONS="LCD Soundsystem|Sisters of Mercy|Drumsound & Bassline Smith|Drumsound and Bassline Smith|O.B.F SOUND SYSTEM"

# Handle detection: names ending with 2+ digits (e.g. user123, vinny141)
HANDLE_REGEX="[a-zA-Z]+[0-9]{2,}$"

echo "Fetching all tracks for label cleaning..."
curl -s "$BASE_URL/music?limit=2000" > all_tracks.json

# Process tracks
jq -r '.[] | select(.artist != null) | .id + "|" + .artist + "|" + .title' all_tracks.json | while IFS='|' read -r id artist title; do
    # Skip exceptions
    if [[ "$artist" =~ $EXCEPTIONS ]]; then
        continue
    fi

    # Determine if suspicious
    is_suspicious=0
    if [[ "$artist" =~ $LABEL_REGEX ]]; then
        is_suspicious=1
    elif [[ "$artist" =~ $HANDLE_REGEX ]]; then
        is_suspicious=1
    fi

    # If artist name looks suspicious
    if [ $is_suspicious -eq 1 ]; then
        # Try different separators: " - ", " – ", " — ", " | ", " ' "
        sep=""
        if [[ "$title" == *" - "* ]]; then
            sep=" - "
        elif [[ "$title" == *" – "* ]]; then
            sep=" – "
        elif [[ "$title" == *" — "* ]]; then
            sep=" — "
        elif [[ "$title" == *" | "* ]]; then
            sep=" | "
        elif [[ "$title" == *" ' "* ]]; then
            sep=" ' "
        elif [[ "$title" =~ " '"[A-Z] ]]; then
            # Special case for "Artist 'Title'" without space before '
            # but we need to be careful. Let's stick to " ' " for now or very specific ones.
            sep=" ' "
        fi

        if [ ! -z "$sep" ]; then
            # Split title: "Real Artist [sep] Real Title"
            # Extract everything before the first occurrence of $sep
            nA=$(echo "$title" | sed "s/$sep.*//")
            # Extract everything after the first occurrence of $sep
            nT=$(echo "$title" | sed "s/^[^-|–—]*$sep//")
            
            # Clean up trailing quotes if we split on " ' "
            if [ "$sep" == " ' " ]; then
                nT=$(echo "$nT" | sed "s/'$//")
            fi

            if [ "$nA" != "$artist" ] && [ ! -z "$nA" ]; then
                echo "Fixing Suspicious Artist ($sep):"
                echo "  ID: $id"
                echo "  Old: $artist"
                echo "  Title: $title"
                echo "  New Artist: $nA"
                echo "  New Title: $nT"
                
                # Perform PATCH
                json_payload=$(jq -n --arg artist "$nA" --arg title "$nT" '{"artist": $artist, "title": $title}')
                curl -s -X PATCH "$BASE_URL/music/$id" \
                     -H "Authorization: Bearer $AUTH_TOKEN" \
                     -H "Content-Type: application/json" \
                     -d "$json_payload" > /dev/null
            fi
        fi
    fi
done
