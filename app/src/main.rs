use database::{establish_connection, insert_track};
use spotify::{
    do_auth,
    get_all_tracks,
    COFFEE_IN_THE_MORNING,
    SENT_TO_YOU_WITH_LOVE,
    BANG_YOUR_LINE,
    SZN21, SZN20,
    SZN19, SZN18,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Noiisseur", about = "Options for running Noiisseur.")]
struct Opt {
    /// Do authentication
    #[structopt(short, long)]
    do_auth: bool,

    /// Update playlist tracks 
    #[structopt(short, long)]
    update_tracks: bool
}

fn main() {
    let opt = Opt::from_args();
    if opt.do_auth {
        do_auth();
    }

    if opt.update_tracks {
        let conn = establish_connection();

        let playlist_ids = [
            COFFEE_IN_THE_MORNING, SENT_TO_YOU_WITH_LOVE,
            BANG_YOUR_LINE, SZN21, SZN20, SZN19, SZN18];

        for id in playlist_ids.iter() { 
            let tracks = get_all_tracks(id);
            // TODO: Make this an upsert
            for track in tracks {
                insert_track(&conn, &track.name, &track.url);
            }
        }
    }

    // Select a random song to post to Twitter

}
