# anterofit [![Build Status](https://travis-ci.org/abonander/anterofit.svg?branch=master)](https://travis-ci.org/abonander/anterofit) [![On Crates.io](https://img.shields.io/crates/v/anterofit.svg)](https://crates.io/crates/anterofit)

Anterofit is a collection of Rust macros coupled to a lightweight, self-contained HTTP framework that
allows you to create strongly-typed Rust wrappers around REST APIs with ease.

Inspired by [Square's Retrofit](https://sqaure.github.io/retrofit), as referenced in the name, Anterofit is even
more strongly typed as everything that is feasible to check at compile-time, is. Runtime errors are,
with few exceptions, reserved for error conditions that can only be discovered at runtime.

Usage
-----

###[Documentation](https://docs.rs/anterofit)

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
        <td><code>rustc-serialize</code></td>
        <td>Serde</td>
    </tr>
    <tr>
        <td>Pros</td>
        <td>
            <ul>
                <li> <code>#[derive]</code> on Stable channel </li>
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
                <li> <code>#[derive]</code> requires nightly/unstable feature </li>
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
