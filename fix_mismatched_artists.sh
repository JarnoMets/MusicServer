#!/bin/bash
AUTH_TOKEN="REDACTED_AUTH_TOKEN"
BASE_URL="https://music.jarnomets.com/api"

echo "Fetching all tracks for mismatched artist cleanup..."
curl -s "$BASE_URL/music?limit=2000" > all_tracks.json

# Process tracks
jq -r '.[] | select(.artist != null and .title != null) | .id + "|" + .artist + "|" + .title' all_tracks.json | while IFS='|' read -r id current_artist title; do
    
    if [[ "$title" == *" - "* ]]; then
        prefix="${title%% - *}"
        suffix="${title#* - }"

        if [ -z "$prefix" ]; then continue; fi

        is_suspicious=0
        
        # Case insensitive equality check
        low_current=$(echo "$current_artist" | tr '[:upper:]' '[:lower:]')
        low_prefix=$(echo "$prefix" | tr '[:upper:]' '[:lower:]')

        if [ "$low_current" == "$low_prefix" ]; then
            if [ "$current_artist" != "$prefix" ]; then
                # Case mismatch - always fix for consistency
                is_suspicious=1
            else
                # Exact match - skip
                continue
            fi
        else
            # Mismatch - check heuristics
            if [[ "$current_artist" =~ [0-9]{3,}$ ]] || [[ "$current_artist" =~ [♪\(\)\[\]] ]] || [[ "$current_artist" == *.* ]]; then
                is_suspicious=1
            fi
            
            KEYWORDS="(Music|Records|TV|Channel|Productions|Label|Promo|Network|Sounds|Sound|Vevo|Official|Topic|Collective|Beat|Daily|Group|Uploader|Upload|Exclusive|Studio|Digital|Network|VEVO|YouTube|Social|Zone|Spot|Selected|Slav|silverdrizzle|YOLOTANKER|Odisho|Darren|MickeyBeam|luminou|Haazzaarrrdd|fr33dO0m|Mitras|Santan|DEAD|coolrusskiy|JumpUp|Zone|Madorasindahouse|Repopulate|Never Happened|Charlotte de Witte|Alan Fitzpatrick|Etienne|rammstein|Lluís|JUC|Bass Ranger|Lyrics|T7T|Shadowline|HATE|HXBass|MARK BLAIR|TopNotchBE|UK Garage Gold)"
            if [[ "$current_artist" =~ $KEYWORDS ]]; then
                is_suspicious=1
            fi

            if [ ${#current_artist} -lt 25 ]; then
                is_suspicious=1
            fi
        fi

        if [ $is_suspicious -eq 1 ]; then
            echo "Fixing Mismatch:"
            echo "  ID: $id"
            echo "  Old Artist: $current_artist"
            echo "  Title Old:  $title"
            echo "  New Artist: $prefix"
            echo "  New Title:  $suffix"
            
            json_payload=$(jq -n --arg artist "$prefix" --arg title "$suffix" '{"artist": $artist, "title": $title}')
            curl -s -X PATCH "$BASE_URL/music/$id" \
                 -H "Authorization: Bearer $AUTH_TOKEN" \
                 -H "Content-Type: application/json" \
                 -d "$json_payload" > /dev/null
        fi
    fi
done
