# Story: scheduled deliveries launch

The plan behind [docs/STORY.md](../../docs/STORY.md) (see [docs/STORY-DETAILS.md](../../docs/STORY-DETAILS.md) for the technical walk-through), as it stands on 2027-07-07, after the
renegotiation.

| File | Owner | In the story it lives in |
|---|---|---|
| `launch.md` | Ingrid Solberg, project manager | the programme repository |
| `backend/backend.md` | Sam Okafor, backend lead | the backend repository |
| `mobile/mobile.md` | Priya Nair, mobile lead | a URL, generated from Jira versions by her script |
| `support/support.md` | Tomás Reyes, support lead | a shared drive |
| `support/board.html` | Tomás's kanban page over `support.md` (not a plan file) | next to his file |
| `support/help-centre/help-centre.md` | Tomás Reyes, worked by agents | a shared drive |

```bash
PMD_TODAY=2027-07-07 pmd check --deep --strict launch.md
PMD_TODAY=2027-07-07 pmd ready launch.md --deep --owner Tomás
```

Tomás's board: `examples/story/support/board.html?today=2027-07-07` (serve the repository root).

Viewer: `viewer/?root=/examples/story/launch.md&today=2027-07-07&cp=1`.
