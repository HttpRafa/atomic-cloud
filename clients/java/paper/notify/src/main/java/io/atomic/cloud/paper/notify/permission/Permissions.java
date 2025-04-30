package io.atomic.cloud.paper.notify.permission;

import io.papermc.paper.command.brigadier.CommandSourceStack;
import lombok.AllArgsConstructor;
import lombok.Getter;
import org.bukkit.permissions.Permissible;
import org.jetbrains.annotations.NotNull;

@AllArgsConstructor
@Getter
@SuppressWarnings("UnstableApiUsage")
public enum Permissions {
    POWER_NOTIFY("atomic.cloud.power.notify"),
    READY_NOTIFY("atomic.cloud.ready.notify");

    private final String permission;

    public boolean check(@NotNull CommandSourceStack sourceStack) {
        return this.check(sourceStack.getSender());
    }

    public boolean check(@NotNull Permissible permissible) {
        return permissible.hasPermission(this.permission) || permissible.isOp(); // TODO: Remove isOp
    }
}
