### Chinese Practice Tool

A tool to help you practice Chinese.

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

#### Features
- Importing copy pasted characters into pleco categories
- Importing characters from pngs into pleco categories (use a model like pdftoppm to turn pdf -> png: pdftoppm -r 300 -png textbook.pdf textbook_png)
- Importing characters from pleco
- Exporting to pleco
- Generating practice sentences to translate based on pleco cateogires.