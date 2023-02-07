# AlderLake and RaptorLake P->E E->P test
For thread bouce back and forth:
```bash
$ cargo run --release
E->P core latency: 65666659323851
P->E core latency: 65666659393830
E->P core latency: 65666659987127
P->E core latency: 65666659998361
E->P core latency: 65666660557043
P->E core latency: 65666660636342
E->P core latency: 65666661130409
P->E core latency: 65666661232638
E->P core latency: 65666661490042
P->E core latency: 65666661544562
E->P core latency: 65666661655052
P->E core latency: 65666662008811
E->P core latency: 65666662138999
P->E core latency: 65666662170493
E->P core latency: 65666662265592
P->E core latency: 65666662301490
E->P core latency: 65666662395445
P->E core latency: 65666662426716
E->P core latency: 65666662518902
P->E core latency: 65666662540890
```
For process cat+shmem back and forth:
```bash
$ cargo test --release --features=process
P->E core latency: 512995
```
and because of 
```bash
error[E0499]: cannot borrow `r` as mutable more than once at a time
   --> src/main.rs:189:9
    |
151 |     let output = fork(
    |                  ---- first borrow later used by call
...
155 |         |_, _| {
    |         ------ first mutable borrow occurs here
...
165 |                 r.receive_trusted(|p: &[u8]| {
    |                 - first borrow occurs due to use of `r` in closure
...
189 |         || {
    |         ^^ second mutable borrow occurs here
...
203 |                 r.receive_trusted(|p| {
    |                 - second borrow occurs due to use of `r` in closure
```
reversely uncomment the sender and receiver, you'll get
```bash
thread 'main' panicked at 'Had unexpected output:
E->P core latency: 43279
', src/main.rs:224:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
[1]+  Done                    clear
```