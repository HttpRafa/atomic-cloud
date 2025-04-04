package io.atomic.cloud.paper.send.permission;

import io.papermc.paper.command.brigadier.CommandSourceStack;
import lombok.AllArgsConstructor;
import lombok.Getter;
import org.bukkit.permissions.Permissible;
import org.jetbrains.annotations.NotNull;

@AllArgsConstructor
@Getter
@SuppressWarnings("UnstableApiUsage")
public enum Permissions {
    SEND_COMMAND("atomic.cloud.command.send");

    private final String permission;

    public boolean check(@NotNull CommandSourceStack sourceStack) {
        return this.check(sourceStack.getSender());
    }

    public boolean check(@NotNull Permissible permissible) {
        return permissible.hasPermission(this.permission) || permissible.isOp(); // TODO: Remove isOp
    }
}
