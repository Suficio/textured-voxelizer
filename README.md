# textured-voxelizer

![Voxelized plane](https://github.com/CheezBarger/textured-voxelizer/blob/master/banner.png)

Generates textured voxel models from OBJ files.
Currently only supports voxelization and simplification for BRS files.

The program operates from the command line, to build it use the following command:

```
    cargo build --release
```

Example usage:

```
    models/dauntless.obj dauntless.brs -s 60 --simplify lossless
```

The program supports two color modes when simplifying: lossless, and lossy. Lossless will prioritize color accuracy, while lossy will prioritize brick count.
