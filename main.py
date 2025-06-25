import click
import git
import tempfile
from openai import OpenAI
from pathlib import Path
import os
from dotenv import load_dotenv

load_dotenv()
OpenAI.api_key = os.getenv("OPENAI_API_KEY")
client = OpenAI()


@click.command()
@click.argument("repo_url")
def generate_doc(repo_url):
    with tempfile.TemporaryDirectory() as tmpdir:
        click.echo(f"Cloning repository {repo_url}...")
        git.Repo.clone_from(repo_url, tmpdir)
        click.echo(f"Analyzing project...")
        files = list(Path(tmpdir).rglob("*.py"))
        click.echo(f"Found Python files: {len(files)}")
        for file in files[:3]:
            click.echo(f"\nProcessing {file.relative_to(tmpdir)}:")
            with open(file, "r") as f:
                code = f.read()
            summary = generate_summary_via_llm(code)
            click.echo(f"{summary}")


def generate_summary_via_llm(code):
    prompt = f"以下是一段Python代码，请生成一个简洁明了的中文文档，描述其功能和使用方法：\n\n{code}\n\n文档："
    response = client.responses.create(
        model="gpt-4.1", input=prompt
    )
    return response.output_text


if __name__ == "__main__":
    generate_doc()