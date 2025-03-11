#!/bin/bash

# rustc and run by github-copilot

# 引数が指定されていない場合はエラーメッセージを表示して終了
if [ -z "$1" ]; then
  echo "Usage: $0 <path/to/file.rs>"
  exit 1
fi

# ファイルパスを取得
FILE_PATH="$1"

# ファイル名を取得（ディレクトリ名と拡張子を除く）
FILE_NAME=$(basename "$FILE_PATH" .rs)

# ファイルが存在するか確認
if [ ! -f "$FILE_PATH" ]; then
  echo "File not found: $FILE_PATH"
  exit 1
fi

# rustcでコンパイルして実行ファイルを生成
rustc "$FILE_PATH" -o "$FILE_NAME".out

# コンパイルが成功した場合のみ実行
if [ $? -eq 0 ]; then
  ./"$FILE_NAME".out
else
  echo "Compilation failed."
  exit 1
fi
