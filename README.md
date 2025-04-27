# llm-tui-rs
* TUI application written in Rust to chat with different AI assistants
## features:
* configurable models, different backends with Gemini, OpenAI, gro(q,k), Claude, Phind, Ollama
* creating / renaming chats, no branch functionality yet...


## todo
- [x] fix keys conflicting, enter on name change both sends the prompt and creates a chat
- [x] make the chat title change depending on the chat content like open web ui
- [x] markdown rendering
- [x] MAKE iT SCROLLABLE PROPERLY!!! omg fuck
- [ ] show a dropdown of available models per backend instead of typing in
- [ ] image rendering...
- [ ] make the settings ui more intuitive
- [x] async send the messages,
- [ ] make the assistant message box say the model name
- [ ] attach files and images somehow, maybe slash commands
- [ ] tool calling with browsers, MCP possibly
- [ ] refactor all before doing this???
