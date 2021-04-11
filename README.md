# Noiisseur

Noiisseur is a Twitter bot that tweets a new song daily from one of my playlists. It is linked to the
Twitter account of the same name: [@noiisseur]

## Quick start

## Guide

Noiisseur is implemented in Rust. Data is stored in a SQLite database. It uses [diesel] as an ORM for
interfacing with the database. Noiisseur exposes a cli to interact with the application. The commands
are discussed later in the guide.

### Setup

Setting up Noisseur currently requires some ceremony. This will be improved in future versions of the
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
