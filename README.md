# Logic solver

Experimental interpreter that is able to evaluate propositional logic statements.

```bash
$ echo "
p := 1
q := 0
~p v ~q <=> ~(p ^ q)" > statement.prop
$ cargo run statement.prop

Result: true
```

## Visualizing AST

It's possible to draw a graphical representation of the Abstract Syntax Tree used
as immediate representation of the statement.

```bash
$ cargo run statement.prop && dot -Tsvg graph.dot -o graph.svg
```

![visualization of graph](./graph_murphy.svg)
