# Inline Drafting Assistant

Source item: affectUI Pro `components/writing-assistant` (functional reference only; see README).

## Purpose
A small AI helper that sits inside an editor, CRM or sidebar. A person asks it to rewrite, shorten, change tone, translate or draft text, reads streamed replies in a chat card, and can dictate by voice. Landing pages can use it as a try-it surface.

## Data
| Field | Type | Meaning |
|---|---|---|
| `assistant_name` | string | Name shown at the top. Defaults to "Quill". |
| `assistant_tagline` | string | Line beneath the name. |
| `greeting_text` | string | Line on the welcome view. |
| `prompt_chips` | string[] | Each chip posts its own text as a message on activation. |
| `input_hint` | string | Placeholder of the prompt field. |
| `reply_source` | (message) => string or Promise<string> | Produces the reply for a message. A sample responder is used when omitted. |
| `speech_mode` | `"auto" \| "simulate"` | `"auto"` relies on the browser's recognizer when one exists; `"simulate"` always plays a canned take. |
| `sample_dictation` | string, optional | Sentence used for simulated dictation. |

## Anatomy
In reading order:
1. Welcome view: assistant name, tagline, greeting, and the prompt chips.
2. Conversation: messages in order, the user's messages and the assistant's replies; a "writing" line with a label while a reply is pending.
3. Prompt field: a multi-line text field that grows to a few lines, then scrolls.
4. Controls at the bottom right of the card: microphone toggle, send, and a new-chat control.

## States
- **Welcome:** greeting and chips shown; no conversation yet.
- **Chatting:** the welcome view hands over to the conversation with a short rise.
- **Waiting for reply:** the "writing" line shows a moving placeholder until the reply starts.
- **Streaming reply:** reply text appears word by word.
- **Listening:** the microphone control takes an active treatment with calm level bars and a breathing ring; words appear live in the field.
- **Dictation finished:** the finished sentence moves into the conversation as the user's message.
- **Speech unavailable:** with `speech_mode` set to simulate, or when the browser lacks recognition, a sample take plays instead.
- **Empty field:** send is inactive.

Transitions: a chip or send creates a user message and a pending reply; ending dictation creates a user message; new chat returns to the welcome view.

## Interactions
- **Pointer:** press a chip to send it; type in the field; press send; press the microphone to start and stop dictation; press new chat.
- **Keyboard, Enter:** sends what is in the prompt field.
- **Keyboard, Shift with Enter:** starts a new line.
- **Keyboard, Tab:** moves through the chips, the field, the microphone, send and new chat.
- **Touch:** same as pointer; controls are larger.

## Motion (functional)
- A soft luminance floats behind the greeting and dims after the first message.
- The welcome view hands over to the conversation with a short rise.
- While listening, the microphone control changes shape and a soft ring breathes outward; words appear live in the field.
- Once speech ends, the sentence becomes the user's bubble; a shimmering placeholder marks the wait, then the answer arrives word by word.
- Reduced motion: ambient light and shimmer are frozen; answers render immediately.

## Accessibility
- The microphone is a toggle button whose state is exposed as pressed, and whose label states what pressing it will do next.
- New messages are read out courteously; the placeholder line has its own text label.
- Every action is reachable without a pointer; dictation remains optional.

## Responsive
- One card that fills its container up to a moderate width; the prompt, microphone and send sit at the bottom of the card.
- Dictation relies on the browser's recognizer, matched to the page language; where none exists, a canned take plays.

## Composition hints
- Card for the container, with a soft decorative backdrop.
- Chip buttons for suggestions.
- Chat message list.
- Multi-line text field with send.
- Toggle button for dictation with a level indicator.
- Polite live region.

## Acceptance checks
1. Pressing a prompt chip adds it as a user message and starts a pending reply.
2. Pressing Enter in the field sends its text and clears the field; Shift with Enter inserts a new line.
3. The microphone control reports a pressed state while listening, and its label changes to say what pressing it will do.
4. With `speech_mode` set to simulate, pressing the microphone plays the sample take and creates a user message from it.
5. A reply supplied by `reply_source` appears in the conversation; a rejected promise shows an error message in the conversation and leaves the field usable.
6. The prompt field grows to a few lines and then scrolls internally.
7. With reduced motion requested, the glow and shimmer are static and the reply appears in full at once.
8. Pressing new chat returns the view to the welcome state with the greeting and chips.
</content>
