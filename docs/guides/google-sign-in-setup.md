# Google sign-in: Google Cloud setup

What to create in the Google Cloud console so `apps/api` can run the Google sign-in flow.
The design is [google-sign-in design](../plans/2026-11-26-google-sign-in-design.md). The
code reads the values below from the environment (`.env.example`). No real credential belongs
in the repository.

## 1. Project and consent screen

1. Create or pick a Google Cloud project for the product.
2. **APIs & Services → OAuth consent screen.** User type **External** (the product is for
   people outside one Google Workspace).
3. Fill the consent screen fields:
   - **App name:** the product name as users should see it.
   - **User support email:** a monitored address.
   - **App logo:** optional.
   - **Application home page:** the public site, for example `https://www.<your-domain>`.
   - **Application privacy policy link:** `https://www.<your-domain>/privacy`.
   - **Application terms of service link:** `https://www.<your-domain>/terms`.
   - **Authorised domains:** the registrable domain of the API and the apps, for example
     `<your-domain>`. Google checks every redirect host against this list.
   - **Developer contact information:** an address Google can reach about the project.
4. **Scopes:** add exactly `openid`, `.../auth/userinfo.email`, and `.../auth/userinfo.profile`,
   which are the scopes `openid email profile`. The code requests `openid email profile` and
   reads `email`, `email_verified`, `name` and `picture` from the userinfo response. It requests
   no other scope.
5. **Publishing status:** keep **Testing** until launch and add the founder's and testers'
   accounts under **Test users**. A Testing app accepts only the listed accounts.

## 2. OAuth client

1. **APIs & Services → Credentials → Create credentials → OAuth client ID.**
2. **Application type:** **Web application**.
3. **Name:** one per environment, for example `rust-v2 api (staging)`.
4. **Authorised JavaScript origins:** **leave empty.** The flow is a server redirect from the
   API; the browser never runs Google's JavaScript for this client.
5. **Authorised redirect URIs:** one per environment. The URI is the API's `AUTH_BASE_URL`
   followed by `/callback/google`. `AUTH_BASE_URL` already includes the `/auth` segment, and
   the value must match it exactly, scheme, host, port and path:

   | Environment | `AUTH_BASE_URL` | Redirect URI to register |
   |---|---|---|
   | local development | `http://localhost:4400/auth` | `http://localhost:4400/auth/callback/google` |
   | staging | `https://api-staging.<your-domain>/auth` | `https://api-staging.<your-domain>/auth/callback/google` |
   | production | `https://api.<your-domain>/auth` | `https://api.<your-domain>/auth/callback/google` |

   A redirect URI that differs by one character from the API's value fails at the callback, not
   at the sign-in button. Register each environment's URI in its own client, or list all of them
   in one client.
6. Create. Copy the **client ID** and **client secret** into the environment. Do not paste them
   into chat, a commit, a pull request or a log.

## 3. Environment (per deployment)

| Variable | Value | Notes |
|---|---|---|
| `GOOGLE_CLIENT_ID` | from step 2 | set together with the secret, or neither |
| `GOOGLE_CLIENT_SECRET` | from step 2 | secret store, never the repository |
| `OAUTH_PENDING_SECRET` | 32 or more random bytes | a fresh value per environment; signs the sign-in cookie |
| `OAUTH_RETURN_ORIGINS` | the app origins, comma-separated, scheme and host only | defaults to `CORS_ORIGINS`; a sign-in can return only to these |
| `AUTH_BASE_URL` | the API's public URL with `/auth` | must match the redirect URI registered above |

The API refuses to boot if `GOOGLE_CLIENT_ID` is set without the secret (or the reverse), or if
`OAUTH_PENDING_SECRET` is missing or shorter than 32 bytes.

## 4. Check it

1. Start the API with the environment set. `GET /auth/oauth/google/start?return_to=/` answers
   `302` to `accounts.google.com`, and the response sets the `__Host-rv2_oauth_pending` cookie.
2. Sign in with a test user. The browser lands on the app home, signed in.
3. Sign in with a Google account whose email matches an existing password account that has not
   verified its email. The browser lands on `/login?error=unverified_link` and no session is
   created. This is expected until email verification ships (design, Plan bead 5).

## Who must do what

- **The founder** creates the consent screen and the OAuth client, registers the redirect URIs
  and adds test users. Credentials are not created by the code and are not in the repository.
- **Engineering** sets the environment variables in the deployment and runs the check above.
