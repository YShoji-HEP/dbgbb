dbgbb
===========================
A framework for analyzing debugging data in a Mathematica notebook.

See also [`ArrayObject`](https://github.com/YShoji-HEP/ArrayObject) and [`BulletinBoard`](https://github.com/YShoji-HEP/BulletinBoard).

Highlights
----------
* Read test data from `BulletinBoard` and send debug data to `BulletinBoard` with simple macros.
* The file name, the line number and the column number are automatically retrieved and included in the tag.
* Optional buffered sender to reduce TCP transactions.
* Various tools for data collection: accumuation, oneshot and frequency reduction.
* Debug data can be read during program execution and persist after execution.
* Unsigned/signed integer, real float, complex float and string are supported. For array data, `Vec`, `ndarray` and `nalgebra` are currently supported.

```mermaid
sequenceDiagram
Program->>BulletinBoard: Debugging data
BulletinBoard->>Program: Test data
Program->>BulletinBoard: Debugging data
Notebook->>BulletinBoard: Request
BulletinBoard->>Notebook: Response
Program->>BulletinBoard: Debugging data
Program->>BulletinBoard: Debugging data
```

Example
-------
Before using `dbgbb`, you must set up a [`BulletinBoard`](https://github.com/YShoji-HEP/BulletinBoard) server and set the server address in the environmental variable. It is convenient to set it in `.cargo/config.toml` of your Rust project:
```rust
[env]
BB_ADDR = "ADDRESS:PORT"
```

Rust example:
```rust
use dbgbb::dbgbb;

fn main() {
    let test = vec![1f64, 2., 3.];
    dbgbb!(test);
}
```

The debug data can be read in a Mathematica Notebook. See [`ArrayObject`](https://github.com/YShoji-HEP/ArrayObject) for details.
![Mathematica example](example.png "Mathematica example"){width=500}

ToDo
----
- [ ] Jupyter notebook support (Python, Julia).
- [ ] Windows support. 
- [ ] Support for other arrays.

Q&A
--------------
#### Why not use the `dbg!(...)` macro?
For a small number of variables, it is, in fact, efficient to print them using `dbg!(...)`. However, for a large number of variables like a higher-dimensional array, the output becomes cluttered and difficult to read. Together with a notebook, `dbgbb!(...)` offers an immediate visualization of variables with a similar syntax. In addition, `dbgbb` keeps all revisions in the server, so you can easily compare different versions of code.

#### Why not use a CSV file?
For arrays with more than two dimensions, CSV files are clearly not an option. Also, for large data, the data size is huge compared with `dbgbb` since `dbgbb` stores binary data.
#### Why not use a HDF5 file?
It is sometimes useful to be able to read debugging data while the program is running. HDF5 easily collapses if the file is opened while it is being written. In addition, the syntax of `dbgbb` is much simpler than HDF5, which requires setting the database name, array shape, etc.

#### Why not use an integrated visualizer?
To validate the results, we need the correct data to compare. Such data can be generated in another language like Mathematica or obtained in the literature. Hard-coding long data and plotting it using a visualizer is not effective.
In addition, when the plot is not satisfactory, the entire code must be rerun since all data is gone once the program terminates. Therefore, plotting code and debugging code should be separated.
It is also important to keep the initial erroneus data because otherwise it becomes difficult to quantitatively check improvements. `dbgbb` keeps all versions and the data can be read anytime.
