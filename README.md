# Goofy
Goofy是一个为Qmm-lang编程语言设计的包管理和构建工具.

## 安装
由于Goofy的开发尚未完成, 因此我们暂时无法提供二进制文件。但是你可以通过[源码](https://github.com/miaomiaowu0428/Goofy.git)来安装Goofy.
使用源码构建, 你需要确保你的环境中包含必要的依赖和工具:
- Cargo
- LLVM10  

进入项目目录, 使用
`bash
cargo build --release
`构建Goofy.  
构建完成后, 你可以在`target/release`目录下找到Goofy的可执行文件. 
之后根据你的需要将Goofy的可执行文件添加到你的环境变量中.  
> 例如: 在Ubuntu系统中, 在`~/.bashrc`文件中添加`export PATH=$PATH:/path/to/Goofy/target/release`, 保存并退出.   
使用`source ~/.bashrc`使修改生效.

现在在任意目录使用`Goofy --version`验证安装成功 或者 `Goofy --help`查看帮助信息.

## 使用
开始使用Goofy之前, 你需要确保你的环境中已经包含了llvm和clang.

Goofy提供了一些子命令来帮助你管理你的项目: 

- `new [project_name]`: 使用这个子命令来创建一个新的项目, 你需要将`[project_name]`替换为你的项目名称。

- `build`: 使用这个子命令来构建你的项目. 这将会编译你的代码, 并生成一个可执行文件.

- `run`: 使用这个子命令来运行你的项目, 这将会启动你的应用程序.

以下是一些示例：

创建一个新的项目：
```bash
Goofy new my_qmm_project
```

构建你的项目：
```bash
Goofy build
```

运行你的项目：
```bash
Goofy run
```



