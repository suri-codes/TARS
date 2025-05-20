#import "@preview/cheq:0.2.2": checklist

#show: checklist
#show link: underline
#set heading(numbering: "1.a.1. ")

= TARS (Task Accumulation and Reminder Service)

== New shit!!

- lazygit like ui that allows for fast and efficint management of tasks

- Would like every task to be able to open up as a buffer in helix! that way
  if its more like a general task that you have ideas about you can yap about it in there

- "Directory"-like structure that allows for categorization

- CLI is simply not enough for good UX, just heavily zone in on the TUI interface

- git integration? I'd ideally also want to track projects within tars so i can easily
  manage project stuff, but would i want that stored in the repo, or just linked within
  tars

- what is the architecture going to look like? I want this strictly to be a personal project,
  and my ideal user is just me. 

  - SQLite is definetly on the table no? need to figure out the data layout.

    - need to store groups, tasks, projects?

== Data Layout







== OLD ideas
Really simple to-do app that just takes in notes and stores them globally and then retrieves them when I want to just see them all.

Would be cool to have a tui interface as well.

maybe try to integrate mcp? seems really cool

sqlite for task database

need to really figure out what we want

lowkey gonna just go play warframe gang


= TODO
- [ ] daemon setup for reminding
- [ ] tui setup
- [ ] cli setup
- [ ] canvas lms integration


= A reminding daemon
for later btw
https://github.com/hoodie/notify-rust

= command interface
// what the fuck do i do with this
tars add "work on x and y and something else gang wtf are you doing"


entry {
  group:
  prio:
  description:
  due-by:
}

tars list
prints out all tasks in prio order



^^ this guy fucking lacks vision...

