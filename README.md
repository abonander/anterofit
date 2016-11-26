# anterofit

#### [retrofit](https://square.github.io/retrofit) | *verb*  \ˈre-trō-ˌfit, ˌre-trō-ˈfit\

* *to adapt an existing implement to suit a new purpose or need*

* *a Java library which generates REST API wrappers at runtime using class metadata* 

#### anterofit | *verb (neologism)* \ˈan-tə-(ˌ)rō-ˌfit\

* *to create an original implement to suit a particular purpose or need*

* *a Rust crate which generates type-safe REST API wrappers at compile time using macros*

Anterofit is a collection of Rust macros coupled to a lightweight, self-contained HTTP framework that
allows you to create strongly-typed Rust wrappers around REST APIs with ease.

Inspired by [Square's Retrofit](https://sqaure.github.io/retrofit), as referenced in the name, Anterofit is even
more strongly typed as everything that is feasible to check at compile-time, is. Runtime errors are,
with few exceptions, reserved for error conditions that can only be discovered at runtime.

Usage
-----

With [`rustc-serialize`](https://crates.io/crates/rustc-serialize):

```toml
[dependencies]
anterofit = "0.1"
```

With [`serde`](https://crates.io/crates/serde) and JSON serialization:

```toml
[dependencies.anterofit]
version = "0.1"
default-features = false
features = ["serde", "serde_json"]
```

###Choosing a serialization framework

`rustc-serialize` and Serde both have their pros and cons. Neither is a clear winner over the other; it depends
entirely on your needs.

<table>
    <tr>
        <td />
        <td>`rustc-serialize`</td>
        <td>Serde</td>
    </tr>
    <tr>
        <td>Pros</td>
        <td>
            <ul>
                <li> `#[derive]` on Stable channel </li>
                <li> (Potentially) Faster Compilation:
                    <ul>
                        <li> No Transitive Dependencies </li>
                        <li> Uses compiler datastructures / doesn't have to reparse </li>
                    </ul>
                </li>
            </ul>
        </td>
        <td>
            <ul>
                <li> More/extensible serialization options </li>
                <li> Likely more performant serialization </li>
            </ul>
        </td>
    </tr>
    <tr>
        <td>Cons</td>
        <td>
            <ul>
                <li> Likely less performant serialization </li>
                <li> Still somewhat unstable (may go away or change forms) </li>
                <li> (Useful) serialization limited to JSON </li>
            </ul>
        </td>
        <td>
            <ul>
                <li> `#[derive]` requires nightly/unstable feature </li>
                <li> Slower compilation:
                    <ul>
                        <li> Several transitive dependencies </li>
                        <li>
                            Procedural macros currently have to reparse the token stream instead
                            of reusing compiler datastructures
                        </li>
                    </ul>
                </li>
            </ul>
        </td>
    </tr>
</table>
