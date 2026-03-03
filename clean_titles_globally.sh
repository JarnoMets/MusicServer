#!/bin/bash
AUTH_TOKEN="Baske123"
BASE_URL="https://music.jarnomets.com/api"

echo "Fetching all tracks for global title cleanup..."
curl -s "$BASE_URL/music?limit=2000" > all_tracks.json

# Extract id|title
jq -r '.[] | .id + "|" + .title' all_tracks.json | while IFS='|' read -r id title; do
    # Multi-pass cleaning for various junk tags
    # Handle brackets (square and round) and common YouTube/Promo tags
    cleaned_title=$(echo "$title" | sed -E '
        s/[ \t]*[\(\|][ \t]*(Official Music Video|Official Music Audio|Official Lyric Video|Official Visualizer|Official Visualiser|Official Video|Official Audio|Animated Visualiser|Animated Visualizer|Animated Video|Video Animation|Full Song|High Quality|Music Video|Lyric Video|Audio Video|Visuals? Audio|Music Visual|Visualizer|Visualiser|Visuals?|Audio|Video|HD|HQ|OUT NOW|Free Download|Explicit|Visuals?|YouTube|GRM Daily|Studio Brussel|Monstercat Release|Monstercat EP Release|Monstercat LP Release|Original Mix)[ \t]*[\)]?[ \t]*//gI;
        s/[ \t]*\[[ \t]*(Official Music Video|Official Music Audio|Official Lyric Video|Official Visualizer|Official Visualiser|Official Video|Official Audio|Animated Visualiser|Animated Visualizer|Animated Video|Video Animation|Full Song|High Quality|Music Video|Lyric Video|Audio Video|Visuals? Audio|Music Visual|Visualizer|Visualiser|Visuals?|Audio|Video|HD|HQ|OUT NOW|Free Download|Explicit|Visuals?|YouTube|GRM Daily|Studio Brussel|Monstercat Release|Monstercat EP Release|Monstercat LP Release|Original Mix)[ \t]*\][ \t]*//gI;
        s/^\[(DnB|Drum & Bass|Electro|House|Dubstep|Glitch Hop|Trap|Future Bass|Hardcore|Indie Dance|Nu Disco|Trance)\] - //gI;
        s/\|[ \t]*Official[ \t]*Video//gI;
        s/\|[ \t]*Official[ \t]*Music[ \t]*Video//gI;
        s/\|[ \t]*Official[ \t]*Audio//gI;
        s/\|[ \t]*YouTube//gI;
        s/\|[ \t]*GRM Daily//gI;
        s/\|[ \t]*Studio Brussel.*//gI;
        s/ — Drumcode — .*//gI;
        s/ - Official Video//gI;
        s/ - Official Music Video//gI;
        s/ - YouTube//gI;
        s/ - Monstercat Release//gI;
        s/ - Monstercat EP Release//gI;
        s/ - Monstercat LP Release//gI;
        s/[ \t]*[\(\[]Original Mix[\)\]]//gI;
        s/[ \t]+Original Mix//gI;
        s/\(\)//g;
        s/\[\]//g;
        s/  +/ /g;
        s/^ //g;
        s/ $//g
    ')
    
    if [ "$title" != "$cleaned_title" ]; then
        echo "Updating ID $id:"
        echo "  Old: $title"
        echo "  New: $cleaned_title"
        
        json_payload=$(jq -n --arg title "$cleaned_title" '{"title": $title}')
        curl -s -X PATCH "$BASE_URL/music/$id" \
             -H "Authorization: Bearer $AUTH_TOKEN" \
             -H "Content-Type: application/json" \
             -d "$json_payload" > /dev/null
    fi
done
