# Blaseball Observer 

Watch a game of [blaseball](https://blaseball.com).

The rust version.
Not the random python script I have lying around.

## Usage
```shell script
$ cargo build
$ ./target/debug/blaseball-observer-rs [team nickname]
```

or leave the team blank for the Baltimore Crabs 🦀.

Nicknames with spaces should be quoted: `"jazz hands"`

> Note: Requires libdbus to be installed, for notifications.
>
> On Ubuntu 20.04, try installing the `libdbus-1-dev` package

## Thanks
to SIBR for some [api information](https://github.com/Society-for-Internet-Blaseball-Research/blaseball-api-spec)