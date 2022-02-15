# **vybe**
## *Initial Documentation*

### Description:
Generates a 10 song playlist based of song data submitted by the user.  Currently implemented with a CLI interface.  We hope to turn this into a web app in the future.

### Dependencies:
vybe relies on:
* Rust
* Cargo
* Having a spotify account

### Set-up & Use:

1. Make sure you have rust and cargo installed.
2. Fork and clone the repository.<br/>
*Note: Log in to Spotify in browser if not already logged in.*
3. Cargo run
4. A browser will pop up and direct you to a "broken" web page. Copy and paste the URL into the CLI to grant authorization. 
5. You will be prompted to name your vybe (this creates an empty playlist)
6. You will be prompted for an artist name.  Select the number that matches your intended artist.
7. You will be given a list of 10 most popular songs by that artist. Select the number of song you want to use to generate playlist.<br/>
*Note: Sorry, the Spotify API limits what songs we can return by artists. It's trash. Booo. Tomato, tomato.*
8. If there are more than 3 genres to that artist/song, you can manually select which 3 genres to include in the search.
9. If successful, you will receive an Ok response. Check your Spotify for the playlist.<br/>
*Note: If not successful, it's totally your fault. Not us. Ha ha. We're Flawless and Pwofessional Wustaceans. Imperfect code does not exist in our realm.*

# 🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀🦀