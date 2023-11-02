## How to run the program:

NB! You should probably compile this before running, look at the section "How to compile" for more information. Alternatively you can run the compiled version of the program, look at the section "How to run the program without doing all that" for more information.

Run the following command in the terminal:
`./run <number of nodes>`
This will run the number of nodes and connect them to each other. It will use the 61021 as the port.
To stop the program you have to run the following command in the terminal:
`/share/ifi/cleanup.sh`
And connect again and run the script with the other desired amount of nodes

## Docstrings:
To generate docstrings you have to run the following command in the terminal:
`cargo doc --no-deps --open`


## How to compile:

To compile this rust program you need to have rust installed on your computer. You can install rust by following the instructions on the [rust website](https://www.rust-lang.org/tools/install). But you also need rustup and have rustc 1.72 or higher installed.

Before you install rust you have to ssh to the  cluster node c7-1 where you want to run the starting script.

You can install rust by running the following command in the terminal on the cluster node c7-1:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  
```

After you have installed rust you have to fix so you can use rustup and rustc 1.72 or higher. You can do this by running the following command in the terminal:

```bash
nano ~/.bashrc
```

Then you have to add the following line to the end of the file:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Then you have to save the file and exit nano. Then you have to run the following command in the terminal:

```bash
source ~/.bashrc
```

Then you have to run the following command in the terminal to verify that rustup is installed:

```bash
rustup --version
```

Then you have to run the following command in the terminal to verify that rustc 1.72 or higher is installed:

```bash
rustc --version
```

Then you have to run the following command in the terminal to compile the program:

```bash
cargo update && cargo build --release
```

Then you have to run the following command in the terminal to run the program:

```bash
./start_server <number of nodes>
```

## How to run the program without doing all that:
There is a compiled release version of the program in the folder "target". You can run the program by running the following command in the terminal:

```bash
./start_server <number of nodes>
```

 and you clean all up, you have to navigate to where the /share/ifi/cleanup.sh file is and run it.
