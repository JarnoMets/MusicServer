import requests
import json

base_url = "https://music.jarnomets.com/api"
token = "REDACTED_AUTH_TOKEN"
headers = {"Authorization": f"Bearer {token}"}

def get_artists():
    resp = requests.get(f"{base_url}/artists", headers=headers)
    return resp.json()

def set_artist_genre(artist, genre):
    # The endpoint is /api/artists/{artist}/genre
    # But wait, looking at my routes.rs:
    # .route("/artists/{artist}/genre", web::put().to(routes::set_artist_genre_handler))
    # Note: it's under /api, NOT /api/admin
    resp = requests.put(f"{base_url}/artists/{requests.utils.quote(artist)}/genre", 
                        headers=headers, 
                        json={"genre": genre})
    return resp.status_code

weird_patterns = [
    "alias of",
    "appears on",
    "band founded by",
    "Belgian DJ",
    "Belgian techno dj",
    "British electronic musician",
    "Collaboration between",
    "Copenhagen techno producer",
    "DJ/producer",
    "dnb producer",
    "drum and bass producer",
    "drum & bass artist",
    "Electronic artist",
    "Electronic duo",
    "French techno producer",
    "German electronic music",
    "Hair metal band",
    "Hungarian band",
    "mashup artist",
    "moniker of",
    "Slovenian house",
    "techno producer",
    "UK drum & bass",
    "UK electronic",
    "UK techno producer",
    "US band",
    "key tracks",
    "3D!t -",
    "aka Dr. Peet",
    "ragga style vocalist"
]

def clean_weird_genres():
    artists = get_artists()
    cleaned_count = 0
    for artist in artists:
        name = artist.get("name")
        genre = artist.get("genre")
        if not genre or genre == "Unknown":
            continue
            
        is_weird = False
        for pattern in weird_patterns:
            if pattern.lower() in genre.lower():
                is_weird = True
                break
        
        # Also check for very long genres which are usually biographies
        if len(genre) > 30 and not "," in genre: # aliases are usually comma separated or just long
             is_weird = True
             
        if is_weird:
            print(f"Cleaning weird genre for {name}: '{genre}' -> Unknown")
            status = set_artist_genre(name, "Unknown")
            if status == 200:
                cleaned_count += 1
            else:
                print(f"Failed to clean {name}: status {status}")
                
    print(f"Total artists cleaned: {cleaned_count}")

if __name__ == "__main__":
    clean_weird_genres()
