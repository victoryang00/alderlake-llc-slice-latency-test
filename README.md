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
For process cat+shmem back and forth(WIP):
```bash
$ cargo test --release --features=process

```