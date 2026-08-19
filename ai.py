from openai import OpenAI

client = OpenAI(
  base_url = "https://nim.api.nvidia.com/v1",
  api_key = "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"
)

completion = client.chat.completions.create(
  model="Qwen/Qwen2.5-Coder-32B-Instruct",
  messages=[{"role":"user","content":"test"}],
  temperature=0.5,
  top_p=1,
  max_tokens=1024,
  stream=True
)

for chunk in completion:
  if chunk.choices[0].delta.content is not None:
    print(chunk.choices[0].delta.content, end="")

