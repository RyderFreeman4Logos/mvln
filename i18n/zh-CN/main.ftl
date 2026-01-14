# mvln - 简体中文翻译

# 操作消息
op-moving = 移动 { $src } -> { $dest }
op-linking = 创建软链接 { $link } -> { $target }
op-complete = 完成: 移动了 { $files } 个文件, 创建了 { $links } 个软链接
op-dry-run = [预览模式] 未做任何更改

# 等效命令（调试输出）
cmd-mv = mv { $src } { $dest }
cmd-ln = ln -s { $target } { $link }

# 错误消息
err-source-not-found = 错误: 源文件不存在: { $path }
err-dest-exists = 错误: 目标已存在: { $path }
    .hint = 使用 -f/--force 覆盖
err-is-directory = 错误: { $path } 是目录
    .hint = 使用 -d/--whole-dir 移动目录, 或使用通配符 (如 { $path }/*)
err-symlink-failed = 错误: 创建软链接失败 { $link } -> { $target }
    .reason = 原因: { $reason }
err-move-failed = 错误: 移动失败 { $src } -> { $dest }
    .reason = 原因: { $reason }
err-copy-failed = 错误: 复制失败 { $src } -> { $dest }
    .reason = 原因: { $reason }
err-remove-failed = 警告: 文件已复制但无法删除源文件: { $src }
    .reason = 原因: { $reason }
    .note = 文件在两个位置都存在, 可能需要手动清理

# 恢复消息
recovery-header = 文件已移动到: { $dest }
recovery-command = 恢复命令 (回滚用):
recovery-mv = mv { $dest } { $src }

# 帮助文本
help-source = 源文件或通配符模式
help-dest = 目标路径
help-force = 覆盖已存在的目标
help-whole-dir = 整体移动目录 (默认: 遇到目录报错)
help-relative = 使用相对路径软链接 (默认)
help-absolute = 使用绝对路径软链接
help-dry-run = 只打印命令不执行
help-verbose = 详细输出
