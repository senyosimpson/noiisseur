# Noiisseur

Noiisseur is a Twitter bot that tweets songs from a collection of playlists. I use it to tweet a song
daily from one of my playlists to a Twitter account of the same name: [@noiisseur].

## Status

It works. The code is a bit rough and the ergonomics of the application are relatively poor in my opinion.
Nonetheless, I'm leaving it as is. It was a fun small project!

## Guide

Noiisseur is implemented in Rust. Data is stored in a SQLite database. It uses [diesel] as an ORM for
interfacing with the database. Noiisseur exposes a cli to interact with the application. The commands
are discussed later in the guide.

### Setup

Setting up Noiisseur currently requires some ceremony. This will be improved in future versions of the
application. There are two components to the setup: the database and ORM layer and acquiring Twitter
authentication tokens.

#### The database and ORM

As mentioned before, the database used is SQLite. To create a new SQLite

```
sqlite3 database-name.db # The sqlite prompt will open when this is run

sqlite> .database        # Check the database has been created
sqlite> .q               # quit
```

We have to enable foreign key constraints in order for some of our tables to work

```
sqlite3
sqlite> PRAGMA foreign_keys = ON;
```

The ORM requires the diesel cli tool to be installed. We can install it using Cargo.

```
cargo install diesel_cli --no-default-features --features sqlite
```

We need to setup a `.env` file next to work with diesel.

```
echo DATABASE_URL=database-name.db > .env
```

We're now ready to create our tables. We can run the migrations to do so

```
diesel migration run
```

#### Twitter authentication

For Twitter authentication, you need the following:

* Twitter consumer key
* Twitter consumer secret
* Twitter access token
* Twitter access token secret

These should all be set as environment variables in a `.env` file in the root directory.
The names are as stated above but all in capital letters with spaces being replaced by underscores.
For example: `Twitter consumer key -> TWITTER_CONSUMER_KEY`

## CLI API

Perform authentication

```
noi auth
```

Update databases with new tracks (songs) in playlists

```
noi tracks update
```

Post a record to Twitter

```
noi tracks post
```

Adds a new playlist to fetch music from

```
noi playlist add <name> <playlist id>
```

Remove a playlist from the list

```
noi playlist remove <playlist id>
```

[diesel]: https://diesel.rs
[@noiisseur]: https://twitter.com/noiisseur
