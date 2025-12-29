### Chinese Practice Tool

A tool to help you practice Chinese. Integrates with the Pleco app.

#### Usage

To say hello, use the following command:

```bash
cargo run -- greet
```

To generate a example sentence for translation:
```bash
cargo run -- translate 你好
```

Some features require setting OPENAI_API_KEY as an environment variable.

#### Updates
December 28 -> fixing up text -> pleco prompt + adding a new command to generate translations for a single character.

