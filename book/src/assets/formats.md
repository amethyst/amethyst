# Formats

A **format** is a way of encoding the information of an asset so that it can be stored and read later. For example, a texture may be stored as a Bitmap (BMP), Portable Network Graphic (PNG), or Targa (TGA). Game levels can be stored using [RON], [JSON], [TOML] or any other suitable encoding.

Each format has its own strengths and weaknesses. For example, `RON` has direct mapping from the storage format to the in-memory object type. `JSON` is widely used, so it is easy to find a JSON parser in any programming language. `TOML` is easier for people to read.

[json]: http://json.org/
[ron]: https://github.com/ron-rs/ron
[toml]: https://github.com/toml-lang/toml
