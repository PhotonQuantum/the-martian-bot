# 火星救援

[meme](https://zh.moegirl.org.cn/%E7%81%AB%E6%98%9F%28%E7%94%A8%E8%AF%AD%29)

Telegram: [@the_martian_dedup_bot](https://t.me/the_martian_dedup_bot)

Add it to any group and it will notify the sender if a message contains forwards, links, or similar images sent before.

## Self-hosting

1. Create a bot with [@BotFather](https://t.me/BotFather) and get the token.
   Allow it to access arbitrary group messages by `/setprivacy`.
2. Put the token in `.env` file.
    ```dotenv
    TELOXIDE_TOKEN=...
    ```
3. Write `postgresql.conf` to configure the database. There's an example
   in [`postgres/postgresql.conf`](postgres/postgresql.conf).

   You can generate your own at [PgTune](https://pgtune.leopard.in.ua/#/). Don't forget to add `listen_addresses = '*'`
   to the end of the file.
4. Start the bot.
    ```shell
    docker compose up -d
    ```

## Contributing

All contributions are welcome. Feel free to open an issue or submit a pull request if got any idea or problem.

## License

This project is licensed under the [Apache License 2.0](LICENSE.txt).