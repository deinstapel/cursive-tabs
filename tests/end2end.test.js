const { setup } = require('shellshot');

setup();

function cargo_e2e(name, custom) {
    return async () => {
        await expect.command(`cargo build --bin end2end_${name}`)
            .forExitCode(exp => exp.toBe(0));
        await expect.command(
            `tmux new-session -x 80 -y 24 -d 'sh -c "TERM=xterm-256color ../target/debug/end2end_${name}"' \; set status off && sleep 0.05`
        ).forExitCode(exp => exp.toBe(0));

        if (!!custom) {
            await custom();
        }

        await expect.command('tmux capture-pane -J -p -t %0')
            .forStdout(exp => exp.toMatchSnapshot());
        await expect.command('tmux kill-server')
            .forExitCode(exp => exp.toBe(0));
    };
}

it('tests removal of a currently active tab', cargo_e2e('remove_active'));
it('tests removal of a currently inactive tab', cargo_e2e('remove_inactive'));
it('tests switching tabs', cargo_e2e('switch'));
it('tests inserting tabs at defined position', cargo_e2e('add_at'));
it('tests inserting tabs at defined position in panel', cargo_e2e('add_at_panel'));
it('smoke tests the tab panel', cargo_e2e('panel_smoke'));
