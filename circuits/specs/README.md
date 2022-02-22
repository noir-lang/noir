# Aztec 3 Book

## To view the book
### One-off steps

Install mdBook: 

1. [Rust](https://www.rust-lang.org/tools/install): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. [mdbook](https://rust-lang.github.io/mdBook/guide/installation.html#build-from-source-using-rust): `cargo install mdbook`
3. [mdbook-katex](https://github.com/lzanini/mdbook-katex): `cargo install mdbook-katex`

Configure port forwarding, so you can write markdown code on mainframe, but view the book in your local browser:

- `command` + `shift` + `p` to open the command pallette  
- Type and select `Remote ssh open ssh configuration file`  
- Select `/Users/<your name>/.ssh/config` from the drop-down menu  
- Below all your existing `LocalForward` instructions, add another:  
  ```
  LocalForward 3000 <your mainframe IP address>:3000
  ```

The IP address (of the form `_._._._`) to use is the same as all the other LocalForward instructions you already have in your config.
If you already use port `3000` for something, choose something else.

### To view the book

- `cd aztec-internal`  
- `cd markdown/specs/aztec3`  
- `mdbook serve -p 3000 -n <your mainframe IP address of the form _._._._>`  


Open a browser on your local machine and navigate to `localhost:3000`. The book should appear. `mdbook serve` has built the book and is serving the book in 'watch' mode, so any changes you make to the book on mainframe can be viewed by refreshing your browser.

Alternatively, you can use a vscode extension to preview the book's pages (shortcut to previewing is `command`+`shift`+`v`), but it won't be as easily navigable (from page to page) as through the browser.

> **Note** The book is also searchable which is AWESOME (click the magnifying glass when viewing in the browser).

## To edit the book

Just change the files in `src/`.

## What are all the other files in here?

```
book.toml - used by mdbook to build the book
macros.txt - latex macros
sidebar.js & style.css - an add-on to mdbook to show a sidebar of a page's headers, at the right (it's clunky, but mdbook without it isn't nice).
dist/ - the built book, updated each time you save, when running `mdbook serve ...`
```