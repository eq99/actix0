# 使用方法

这个博客项目使用 Rust 语言实现，在进行下面的步骤前，请确保你已安装 Rust 工具链。

第一步：克隆项目到本地。

```shell
git clone https://github.com/xiayulu/actix0.git
```

第二步：进入项目根目录，运行

```shell
cd actix0
cargo run
```

第三步：打开浏览器链接即可 <http://127.0.0.1:8080/>。



# 如何添加博文

你可以在 `blogs` 文件夹下添加 Markdown 文档，首页会以列表形式显示文件名誉修改时间。



# 使用的技术栈

这个博客 APP 使用自己编写的解析器 MarkX 把 Markdown 转化为 HTML 文档。支持如下特性：

- 行内元素：行内代码，行内公式，加粗，自动链接；
- 标题：最多三级标题；
- 代码块：使用 Prismjs 实现语法高亮；
- 数学块：使用 Katex 渲染。

后端采用 Rust 语言 actix-web 框架。

