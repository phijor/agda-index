# agda-index

Extract function names and type definitions from Agda modules rendered to HTML by `agda --html`.

## Why?

I want to find definitions by name in large Agda libraries,
but I don't know how to write proper Agda backends.

## How?

Scan all rendered modules under `html/` for definitions,
select some using a [fuzzy finder](https://github.com/junegunn/fzf)
and open the definition sites in a browser:

```sh
agda-index html/*.html | fzf -d' ' --with-nth='2' | cut -d' ' -f1 | xargs firefox
```

## Licence

This project is subject to the terms of the Mozilla Public License, v. 2.0,
see [LICENSE](./LICENSE).
The [Agda icon](./resources/Agda.svg) has been adapted from the official
Agda logo, and is distributed under the terms of the [Agda license](./resources/LICENSE-Agda.txt).
