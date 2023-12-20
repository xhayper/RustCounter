# RustCounter

多种风格可选的萌萌计数器

![RustCounter](https://pi.hayper.xyz/rust-counter/count/RustCounter.githubformat=png)

<details>
<summary>More theme</summary>

##### asoul

![asoul](https://pi.hayper.xyz/rust-counter/number/1234567890?theme=asoul&format=png)

##### moebooru

![moebooru](https://pi.hayper.xyz/rust-counter/number/1234567890?theme=moebooru&format=png)

##### rule34

![Rule34](https://pi.hayper.xyz/rust-counter/number/1234567890?theme=rule34)

##### gelbooru

![Gelbooru](https://pi.hayper.xyz/rust-counter/number/1234567890?theme=gelbooru&format=png)

##### e621

![e621](https://pi.hayper.xyz/rust-counter/number/1234567890?theme=e621&format=png)

  <details>
    <summary>NSFW</summary>

##### moebooru-h

##### gelbooru-h

  </details>
</details>

## Demo

[https://pi.hayper.xyz/rust-counter/](https://pi.hayper.xyz/rust-counter/)

## Usage

### Install

#### Deploying on your own server

```shell
$ git clone https://github.com/xhayper/RustCounter.git
$ echo 'DATABASE_URL="sqlite://database.sqlite"' >> .env
$ cargo sqlx database create
$ cargo sqlx migrate run
$ cargo run --release
```

### Configuration

`Rocket.motl`

```toml
[default.databases.sqlite_counts]
url = "database.sqlite"
```


## Query

- `theme` - theme you gonna use (default: moebooru)
- `length` - amount of number to show, will automatically expand  if the number is larger than what was set (default: 7)
- `pixelated` - should the svg be rendered with pixelated style? (default: true)
- `format` - choose between `png` and `svg` format (default: svg)

## Credits

- [replit](https://replit.com/)
- [A-SOUL_Official](https://space.bilibili.com/703007996)
- [moebooru](https://github.com/moebooru/moebooru)
- rule34.xxx NSFW
- gelbooru.com NSFW
- e621.net NSFW
- [Icons8](https://icons8.com/icons/set/star)
- [journey-ad](https://github.com/journey-ad/)

## License

[MIT](LICENSE)
