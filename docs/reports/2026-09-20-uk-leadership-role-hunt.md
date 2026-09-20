# UK leadership role hunt, 2026-09-20

Two requirements were set as essential: a working pattern of **at most 25%
in-office**, and a **B2B / outside-IR35** engagement. Roles are ranked by
mandate — whether the holder gets to build a function and teach — with the title
band as a secondary signal.

## The finding

**On the UK's specialist outside-IR35 board, remote and mandate do not co-occur.**

All 30 live listings were fetched on 2026-09-20 and classified by
`crates/rv2-hiring`:

| | Count |
|---|---|
| Listings on the board | 30 |
| Clear both essential filters (remote **and** outside IR35) | 6 |
| Carry a department-building mandate | 1 |
| Both | **0** |

The six that are remote are execution roles: two QA engineers, an IFS
consultant, a RAID lead, and two software engineers. The highest mandate score
among them is one signal — the word "mentor" in a QA posting.

The one listing with a real mandate is **Hybrid**.

This is asserted as a test, `remote_and_mandate_do_not_co_occur_anywhere_on_this_board`,
so a phrasing change cannot quietly reverse it.

## The role worth a call

**Head of Artificial Intelligence** — £900/day, outside IR35, posted 18/09/26 via
VirtueTech Recruitment Group, energy trading organisation, London Area.

<https://outsideir35.org.uk/jobs/head-of-artificial-intelligence/45gZ5Bds8Z>

It is the shape that was asked for, in its own words:

- "leading an AI function within a dynamic energy trading organization embarking
  on an aggressive AI adoption strategy"
- "defining the AI development direction, **shaping the team**, and remaining
  hands-on with technical implementations"
- "**Lead the existing AI team while mentoring and guiding future hires**"
- "Develop and demonstrate **proofs of concept** … bringing AI ideas **from POCs
  to production**"
- "Familiarity with large language models, generative AI, and **autonomous AI
  agents**"

Three mandate signals count: hires-and-grows ("future hires"), research-to-production
("proofs of concept"), owns-department ("strategic direction"). Title band is
head-lead, below the Director / VP / C-level bands named in the Litmus answer —
which is the same tension the Lloyds "Lead Engineer (Advanced AI Engineering)"
role created, and the reason ranking is mandate-first.

**What stands between it and both filters: one number.** The board states the
work type as "Hybrid" with no day count. At one day a week it clears the 25%
ceiling; at two it does not. That is a single question to the recruiter, and it
is the only thing in the way.

Confidence: **rock** on the posting's contents and its IR35 status, which were
read directly. **water** on the office days, which nothing published states.

## The near misses, and what would have to change

| Role | Rate | Blocked by |
|---|---|---|
| Data Lead, London Area | £1,200/day | Hybrid, no day count. Two mandate signals. |
| Lead Integration Architect, London Area | £650/day | Hybrid, no day count. Two mandate signals. |
| Technical Architect, London Area | not stated | Hybrid, no day count. One mandate signal. |

Each is the same blocker as the Head of AI role and the same one question.

## Where the seam is

The outside-IR35 board is the seam, and it is the only source in this sweep that
produced roles clearing the engagement filter at all.

Nothing on an employer applicant-tracking board clears it. That is asserted as a
test too: all 103 roles the register holds from Greenhouse, Lever, Ashby and
SmartRecruiters are permanent postings. A company advertising through its own ATS
is advertising a job it intends to fill with an employee. The register's 101
original companies therefore contribute zero candidates under these two filters —
not because the companies are wrong, but because the boards the register reads are
the wrong kind of board for this requirement.

The two seams worth working next, both of which leave the determination with the
contractor's own company:

- **Companies below the Companies Act small-company thresholds.** They do not owe
  an IR35 determination at all.
- **Companies with no UK entity** engaging a UK-based person B2B. Several
  all-remote employers use this route.

## The sweep that produced this, and what it got wrong

A workflow ran eight blind lanes across contract boards, fractional marketplaces,
UK enterprises standing up AI units, consultancies, remote-first employers, the
Rust ecosystem, the register's own feeds, and teaching-mandate phrasing. 85 agents,
948 tool calls, 13m33s. Each surviving candidate was then attacked by three
independent lenses — remote, engagement, mandate — each instructed to refute and
to default to refuted where evidence was absent.

**Its synthesis was wrong.** It reported the seam as empty and listed fractional
and contractor platforms under "not covered", when two of its own lanes had
searched exactly those and returned nine B2B candidates. The cause was in the
script, not the agents: it bucketed results into full-pass, remote-only and
mandate-only, and had no bucket for *passes engagement, fails remote* — which is
where every outside-IR35 candidate landed. They were dropped unreported.

The numbers above come from reading the board directly afterwards, not from the
workflow's summary.

A second consequence of the refute-by-default instruction: several real listings
were refuted because their URLs could not be fetched. `outsideir35.org.uk`
listings churn within days and the mirror at `contracts.outsidespy.co.uk` returns
403 to a scripted request. A fetch failure is not evidence a role does not exist.

## What this did not cover

- Direct approaches and referrals, which is where fractional engagements are
  usually filled rather than advertised.
- Fractional marketplaces behind a login: GoFractional and FractionalJobs were
  reached only as far as their public listings.
- Venture studios and portfolio-company introductions.
- Any assessment of whether the Head of AI engagement would survive a status
  determination in practice; the board's admission policy is the only evidence
  held for it.
- Rate benchmarking. The rates quoted are the rates advertised.

## How to re-run it

The board capture lives at
`crates/rv2-hiring/tests/fixtures/outside-ir35-board-2026-09-20.json` and the
classifications are `cargo test -p rv2-hiring`. Re-fetching the board and
replacing the fixture will fail
`remote_and_mandate_do_not_co_occur_anywhere_on_this_board` the moment a listing
appears that is both remote and mandate-carrying, which is the result worth
being told about.
