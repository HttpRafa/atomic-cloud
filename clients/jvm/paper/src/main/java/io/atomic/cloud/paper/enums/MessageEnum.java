package io.atomic.cloud.paper.enums;

import lombok.Getter;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.minimessage.MiniMessage;

public enum MessageEnum {
    PREFIX("<color:#50AAFF>ᴀᴄ <gray> ⋆ "),
    INFO_CMD_LINE("<gray><st>--------------------"),
    INFO_CMD_STRING_1("<gray>» <b><gradient:#50AAFF:#23BFFF>Atomic Cloud</gradient></b>  <aqua>☁ "),
    INFO_CMD_STRING_2("<gray>» "),
    INFO_CMD_STRING_3("<gray>» <color:#50AAFF>Client Version <dark_gray>→ <b><color:#23BFFF>{}"),
    INFO_CMD_STRING_4("<gray>» <color:#50AAFF>Controller Version <dark_gray>→ <b><color:#23BFFF>{}"),
    INFO_CMD_STRING_5("<gray>» <color:#50AAFF>Controller Protocol Version <dark_gray>→ <b><color:#23BFFF>{}"),
    SERVER_NOT_READY("<gold>⚠ <red>Marking server as not ready!"),
    TRANSFER_USER_ALL(
            "<aqua>☁ <color:#50AAFF>Requesting to transfer <color:#23BFFF>all users <color:#50AAFF>to new <color:#23BFFF>servers..."),
    TRANSFER_USER_AMOUNT(
            "<aqua>☁ <color:#50AAFF>Transferring <color:#23BFFF>{} users <color:#50AAFF>to <color:#23BFFF>{}..."),
    TRANSFER_USER_FAILED(
            "<gold>⚠ <red>Failed to transfer <color:#FA2E2E>{} users <red>to server! <gray>(<gold>{}<gray>)"),
    TRANSFER_SUCCESS(
            "<aqua>☁ <color:#50AAFF>Successfully submitted <color:#23BFFF>{} <color:#50AAFF>transfer requests to controller."),
    SERVER_STARTING("<green>⬆ <color:#4AE77A>Spinning up server <color:#2DA953>{}!"),
    SERVER_SHUTDOWN("<red>⬇ <color:#C85252>Shutting down server <color:#FA2E2E>{}!");

    @Getter
    private final String template;

    private static final MiniMessage mm = MiniMessage.miniMessage();

    MessageEnum(String template) {
        this.template = template;
    }

    public Component of(MessageEnum prefix, String... param) {
        String message = "";
        if (prefix == null) {
            message = template;
        } else {
            message = prefix.template() + template;
        }
        for (String s : param) {
            message = message.replaceFirst("\\{}", s);
        }
        return mm.deserialize(message);
    }
}
