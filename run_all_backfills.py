import requests
import json
import time

base_url = "https://music.jarnomets.com/api"
token = "REDACTED_AUTH_TOKEN"
headers = {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}

def get_canonical_genres():
    return requests.get(f"{base_url}/admin/genres/canonical", headers=headers).json()

def start_backfill(alias, genre_id):
    resp = requests.post(f"{base_url}/admin/genres/aliases/backfill/start", 
                         headers=headers, 
                         json={"alias": alias, "genre_id": genre_id})
    return resp.json().get("session_id")

def run_backfills():
    genres = get_canonical_genres()
    for genre in genres:
        genre_id = genre["id"]
        genre_name = genre["name"]
        aliases = genre.get("aliases", [])
        
        # Also backfill the name itself in case there's case mismatch or it's in guessed_genre
        all_to_backfill = [genre_name] + aliases
        
        for alias in all_to_backfill:
            print(f"Starting backfill for alias '{alias}' -> '{genre_name}'")
            session_id = start_backfill(alias, genre_id)
            if session_id:
                print(f"  Session started: {session_id}")
            else:
                print(f"  Failed to start backfill for {alias}")
            
            # Small sleep to not overwhelm
            time.sleep(0.1)

if __name__ == "__main__":
    run_backfills()
