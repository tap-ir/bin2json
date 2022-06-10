# bin2json

**bin2json** extract metadata from different binary file format to json.

It can take in input a **file**, a **directory** containing different files, a **disk dump**, or a disk **device**. After processing the data, it will export a **json** file containing all the extracted metadata. 
If a **disk dump** or **device** is provided it will recursively, analyze the partition, the file system, and the different kind of files present on the file system. 

**bin2json** is part of the [TAP](https://github.com/tap-ir/) project and the file type it support is the same as the tap project. (When new parser plugin is added to [TAP](https://github.com/tap-ir/) **bin2json** is updated to include the new plugins).

At time of writting this documentation this is the file type that it support (it can be checked with the **-v** option)

```
exif : Extract EXIF info from file
ntfs : Read and parse NTFS filesystem
mft : Read and parse MFT file
prefetch : Parse prefetch file
partition : Parse MBR & GPT partition
lnk : Parse lnk file
evtx : Parse evtx file
registry : Parse registry file
```

## Release binary

Release binary are available [here](https://github.com/tap-ir/bin2json/releases)

## Building 

To compile it you need to have [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed.

Then :

`cargo build --release`

It will generate the binary in :

`target/release/bin2json` 

If you use tapir-ws (TAPIR workspace) rather than using directly this git repository, the file will be generated in :

`../target/release/bin2json`

You can also run it directly with cargo, example : 

`cargo run --release -- -f file_or_directory_path -o output.json`

### Building with device reading feature : 

To compile it with device reading feature : 

  `cargo build --release --features=device` 

You can check that it was compiled with the feature by running it with the -v option :

```
exif : Extract EXIF info from file
ntfs : Read and parse NTFS filesystem
mft : Read and parse MFT file
prefetch : Parse prefetch file
partition : Parse MBR & GPT partition
lnk : Parse lnk file
evtx : Parse evtx file
registry : Parse registry file
device : Mount a device
```

## Running 

Usage  :

```
  USAGE:
      bin2json [FLAGS] [OPTIONS]

  FLAGS:
      -h, --help       Prints help information
      -v, --plugins    List embedded plugins
      -V, --version    Prints version information

  OPTIONS:
      -c, --config <FILE>      Config file path
      -d, --device <DEVICE>    Path to a device to parse
      -f, --file <FILE>        Path to the files to parse
      -o, --output <OUTPUT>    Output file
```

To run bin2json you need to have the file `bin2json.toml` in the same directory than the binary or provide the option (`-c`) `--config` with the path to the configuration file. 

the `--file` argument can point to a directory containing different files (collected by a triage tool for example), a single file, or disk a dump


### Running with logging information

  To show debug information you must run it with the env variable RUST\_LOG set to 'warn' or 'info' depending of the level of information you want to be shown. 

On Linux or Mac OS X : 

`RUST_LOG=info ./bin2json -f file_or_directory_path -o output.json` 

# Configuration 

The `bin2json.toml` file contain a map of plugin and data type. Bin2json will detect the type of the file and if it contain a compatible plugin it will run it against the file to extract the metadata. 

```
[plugins_types]

ntfs = ["filesystem/ntfs"]
mft = ["filesystem/mft"]
partition = ["volume/partition"]
exif = ["image/jpeg", "image/png", "image/tiff"]
lnk = ["windows/lnk"]
prefetch = ["windows/prefetch"]
evtx = ["windows/evtx"]
registry = ["windows/registry"]
```

If you don't want to run some of the plugins you can comment or remove the one that you don't want.
For example to avoid executing the evtx and registry plugin : 

```
[plugins_types]

ntfs = ["filesystem/ntfs"]
mft = ["filesystem/mft"]
partition = ["volume/partition"]
exif = ["image/jpeg", "image/png", "image/tiff"]
lnk = ["windows/lnk"]
prefetch = ["windows/prefetch"]
#evtx = ["windows/evtx"]
#registry = ["windows/registry"]
```

## Help

We will answer your questions on [Discord](https://discord.gg/C8UdFG6K)

## License

The contents of this repository is available under Affero GPLv3 license.
