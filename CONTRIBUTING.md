# Contributing

If you are able to commit directly to master, don't; doing so will most likely have all your changes rolled back via a force push.
Please use merge requests to do any modifications.

Here's the basics on how to do this:

```bash
# Fork the main repo using the fork buttom on the top right.

# Clone the fork into your PC
$ git clone https://github.com/<your-organization>/PolyMod.git

# CD to the repository
$ cd PolyMod

# Create a new branch based on the master branch
# change the new_branch_name part to something that defines better the change you are making.
$ git fetch https://github.com/PolyTech-Modding/PolyMod.git main:new_branch_name
# Switch to that new branch
$ git switch new_branch_name

# *make the changes you want*

# Commit all the changes to that branch
$ git commit .
# and push the branch with the changes to github
$ git push
```

Now you can do a PR to PolyMod using that branch.
While the PR is going, reviewers will likely make changes or ask for them, you can simply do those modifications, commit and push them, and the PR will automatically see them.

If the PR doesn't pass the CI/CD checks, it won't be merged until fixed.

## Code Style

We don't follow most formatting applications because it often produces unreadable results.
There are some exceptions to this rule, such as `rustfmt`, but for exmple, `tidy` is not used for HTML.

Generally, there are a few rules to note, the rest should be grokable from existing rules:

Add an empty line before and after logical blocks, but only if there is code before or after it. For example:

```rust
fn foo() {
    let x = true;

    if x {
        println!("x is true");
    }

    let y = 1u64;

    match y {
        1 => println!("y is 1"),
        other => println!("y is not 1, it is {}", other),
    }
}
```

Add an empty line after the subject line in documentation and comments.
For example:

```rust
/// This is the subject.
///
/// This is more detailed information.
///
/// Note the empty line after the subject, and between paragraphs.
fn foo() { }
```

For consistency sake, use spaces instead of tabs everywhere.
In JS and Rust, there must be 4 spaces in the indentation, and in HTML and CSS, 2 spaces.
