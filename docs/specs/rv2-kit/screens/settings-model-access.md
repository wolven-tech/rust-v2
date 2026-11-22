# Settings: Model Access

Source item: affectUI Pro `screens/settings-models` (functional reference only; see README).

## Purpose
An administrator decides which AI models people in the workspace may use: the default model for chat and for images, each provider's models switched on or off along with who may use each, and the organisation's own provider keys and custom endpoints.

## Data
| Field | Type | Meaning |
|---|---|---|
| provider_list | { id; display_name; hosting_region: "eu" \| "global" \| "custom"; logo_url?; model_list: { id; display_name; model_kind: "chat" \| "image"; is_enabled: boolean; access_level: "everyone" \| "engineering" \| "admins" }[] }[] | Every provider with its models. |
| default_chat | string | A model id, or "auto". Auto shows a short explainer beneath it. |
| default_image | string | Image model id. |
| api_keys | { provider_id: string; masked_value: string; added_at: string }[] | Workspace API keys, already masked. |
| filter | "all" \| "chat" \| "image" \| "custom" | Model-kind filter. |
| query | string | Search across providers and models; a match opens its provider. |
| is_loading | boolean | Defaults, providers, and keys shown as secondary shapes. |
| is_empty | boolean | No models configured: the defaults sit on Auto; a no-models message with a custom-model button; a no-keys message with an add-key button. |
| on_save | (state) => void | Called on every change. A default that has been switched off falls back: chat falls back to Auto. |
| on_navigate | (label: string) => void | Settings navigation. |

## Anatomy
Reading order:
1. Settings frame: navigation with Models marked current.
2. Page header: the title "Models", a one-line description.
3. Defaults card: the chat model selector with a line on what it is used for; the Auto explainer beneath it when Auto is chosen (a short description of how Auto picks a model and how much it reasons); the image model selector with its own line.
4. Provider list card: filter tabs (All, Chat, Image, Custom), a search field, a custom-model button, then provider cards. Each provider card shows its logo, display name, region label with a description, and a count of enabled models. Cards start collapsed and open on a search match. Inside: model rows, each with its name, a hosting note, an access menu, and an on/off switch. Region groups are ordered EU-hosted first, then global, then custom.
5. API keys card: an explanation that a provider's own key bills that provider directly; a list of masked keys with their added date; an add-key button. An empty state appears when there are none.

## States
- **Loaded.** The default.
- **Loading.** Defaults, provider cards, and keys shown as secondary shapes.
- **Provider collapsed or expanded.** The provider card toggles; the expanded state is announced.
- **Model off.** The row is muted. If it was a default, the default falls back: chat to Auto; image to the first image model still on; if none remains, the default reads "none".
- **Search active.** Matching providers open; non-matching providers hide; a count shows the matches.
- **Add custom model or key form open.** An inline form with name, endpoint, or key; the key field is masked.
- **Access menu open.** Choices of who may use the model.
- **Auto explainer open.** Shows its short text beneath the Auto choice.

## Interactions
- The chat and image selectors open a menu of enabled models; Enter chooses; Escape closes.
- Choosing Auto shows the explainer.
- The model switch toggles on or off with a click or Space; the access menu opens from its own control.
- Filter tabs: arrow keys move; Enter selects.
- Search: typing opens matching providers; Escape clears it.
- Custom model and add-key forms: Enter adds; Escape cancels.
- Provider card header: Enter or Space toggles it open.

## Motion (functional)
- The Auto explainer unfolds beneath the Auto choice.
- Provider cards open and close in place.
- A decorative, rolling list of model names may sit beside Auto. It stops under reduced motion, and a screen reader hears one sentence describing it.

## Accessibility
- Provider cards are buttons with an expanded state.
- Each model switch has the switch role and is named with its model; each access menu is labelled with its model name.
- The key field is a password field.
- Filter tabs form a tab list.
- The search result count is announced.

## Responsive
- When the container is narrow, each control drops below its label, the filter gets its own row, and the Auto options appear under the explainer.
- The screen fills its container.

## Composition hints
- Settings frame.
- Model selector (menu of enabled models); explainer block.
- Accordion of provider cards with switch rows and an access menu.
- Tab-strip filter; search field.
- Masked key list; inline add form with password field.
- Empty state; skeleton shapes.

## Acceptance checks
1. Switching off the default chat model changes the default to Auto and calls `on_save` with that fallback.
2. Toggling a model's switch calls `on_save` with its enabled value changed.
3. Searching for a model name opens its provider card and hides providers with no match.
4. Adding a custom model with Enter calls `on_save` with the new model; Escape cancels without saving.
5. The key field masks its input.
6. Provider cards list EU-hosted providers before global and custom ones.
7. A model's access menu offers the three access choices and saves the one chosen.
8. Choosing Auto shows the explainer text beneath the chat selector.
9. With reduced motion on, the rolling list beside Auto does not move.
10. Below a narrow width, the filter tabs take their own line, and no horizontal page scroll occurs.
