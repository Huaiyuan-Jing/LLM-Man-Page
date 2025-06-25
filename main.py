import click
import git
import tempfile
import shutil
from openai import OpenAI
from pathlib import Path


@click.command()
@click.argument("repo_url")
def generate_doc(repo_url):
    with tempfile.TemporaryDirectory() as tmpdir:
        click.echo(f"Cloning repository {repo_url}...")
        git.Repo.clone_from(repo_url, tmpdir)
        click.echo(f"Analyzing project...")
        files = list(Path(tmpdir).rglob("*.py"))
        click.echo(f"Found Python files: {len(files)}")
        for file in files[:3]:  # 限制文件数目，便于演示
            click.echo(f"\nProcessing {file.relative_to(tmpdir)}:")
            with open(file, "r") as f:
                code = f.read()
            # 调用 LLM（例如OpenAI API）生成文档（此处用伪代码）
            summary = generate_summary_via_llm(code)
            click.echo(f"{summary}")


def generate_summary_via_llm(code):
    prompt = f"以下是一段Python代码，请生成一个简洁明了的中文文档，描述其功能和使用方法：\n\n{code}\n\n文档："
    response = openai.ChatCompletion.create(
        model="gpt-4o", messages=[{"role": "user", "content": prompt}], temperature=0.3
    )
    return response.choices[0].message.content.strip()


if __name__ == "__main__":

    generate_doc()
