# 仓库命名与元信息

建仓库时可直接照抄本文件的内容到 GitHub / Gitee 的对应字段。

## 推荐名称

| 项目      | 内容                          |
| ------- | --------------------------- |
| 仓库名     | `wechat-article-snippets`   |
| 中文名     | 公众号代码块收录                    |
| 简称 / 别名 | 公众号代码摘录库、WAS                |
| 一句话定位   | 给散落在公众号文章里的代码块，建一份带出处的可检索索引 |

**为什么是它**：`wechat` 明确平台、`article` 明确来源、`snippets` 明确收录物是片段而非整篇转载；全小写连字符符合仓库命名惯例，GitHub 搜索友好，也不会像 `wx` / `mp` 那样被误解为微信小程序。

## 备选名称（含取舍）

| 名称                            | 优点        | 取舍                        |
| ----------------------------- | --------- | ------------------------- |
| `wechat-article-snippets`     | 语义最准确、无歧义 | 略长（推荐）                    |
| `wxcode-archive`              | 短、好记      | `wx` 易与小程序混淆，`archive` 偏冷 |
| `mp-code-index`               | 短、突出索引    | `mp` 在微信语境下多指小程序          |
| `gzh-codebook`                | 中文圈一眼看懂   | 拼音缩写对国际协作者不友好             |
| `wechat-tech-snippets`        | 强调技术向     | 会排除非技术类文章里的代码             |
| `official-account-code-index` | 表述最严谨     | 过长，不利于传播                  |

## 简介 / Description

**中文（推荐填入 GitHub About）**

> 收录微信公众号技术文章中的代码块与示例片段：按原文出处归档，附标题、作者、链接与可运行性标注，方便检索、对照与复用。

**中文短版（Gitee / 目录卡片，40 字内）**

> 按出处归档公众号文章里的代码块，带元信息、可检索、可追溯。

**英文**

> A source-attributed archive of code blocks found in WeChat Official Account articles — indexed by article, author and topic for easy lookup, comparison and reuse.

**英文短版**

> Curated, source-attributed code snippets from WeChat Official Account articles.

## 关键字 / Topics

**GitHub Topics（直接填入设置页，最多 20 个，建议用前 10 个）**

```
wechat
wechat-official-account
weixin
mp-weixin
code-snippets
snippets
code-archive
code-index
tech-articles
chinese-tech
knowledge-base
reading-notes
curated-collection
source-attribution
```

**中文关键字（用于 README、公众号介绍、内部检索标签）**

```
微信公众号、公众号、代码块、代码片段、代码示例、技术文章摘录、
文章索引、代码归档、可追溯出处、原文链接、按语言检索、阅读笔记、
Python、JavaScript、Go、SQL、配置片段
```

**建议的标签维度（条目级 `tags` 字段）**

- 语言 / 技术栈：`python`、`javascript`、`go`、`sql`、`shell`、`yaml`
- 主题：`并发`、`爬虫`、`性能优化`、`部署`、`算法`、`可视化`
- 形态：`完整示例`、`配置片段`、`单行技巧`、`伪代码`

## README 首屏 Slogan 备选

1. 把散落在公众号文章里的代码块，整理成可检索、可复用、可追溯的索引。（当前采用）
2. 公众号里的代码，不该只能截图收藏。
3. 每段代码都有出处：公众号技术文章代码摘录索引。

## 建仓库时的设置建议

- 默认分支 `main`；About 填上面的**中文简介**，主页填仓库 URL（若有文档站）。
- Topics 填前 10 个英文关键字；**不要**加 `awesome-list`，本仓库是摘录库而非链接清单。
- 开启 Issues 与 Discussions：Issues 用于「申请移除 / 报告链接失效」，Discussions 用于收录讨论。
- 关闭 Wiki，避免元信息分散；`INDEX.md` 作为唯一总索引。
- 添加 `LICENSE` 与 `NOTICE.md`，并在 README 首屏说明「代码片段不适用仓库许可证」。
- 创建前先搜一次同名仓库；若被占用，优先改用备选 `wechat-tech-snippets`。
- 建议加 PR 模板，把「原文链接、许可情况、是否已脱敏」设为必填项。
