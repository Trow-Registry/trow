If you want to get VSCode working(ish):

 - Use VSCode stable (not insiders for some reason)
 - Put:
{
    "rust-client.channel": "nightly"
}

In workspace settings. 
 - Cross fingers.

If you have problems, it may be because your nightly doesn't have RLS support,
in which case you need to use an older build
https://github.com/rust-lang-nursery/rls-vscode/issues/181#issue-269383659

