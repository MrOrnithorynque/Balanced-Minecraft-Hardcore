package github.mrornithorynque.bmh.selftest;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import com.mojang.authlib.GameProfile;

import io.netty.channel.embedded.EmbeddedChannel;
import net.minecraft.network.Connection;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.PacketFlow;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket;
import net.minecraft.world.item.ItemStack;
import net.minecraft.core.Holder;
import net.minecraft.core.component.DataComponents;
import net.minecraft.core.registries.Registries;
import net.minecraft.world.item.component.BundleContents;
import net.minecraft.world.item.enchantment.Enchantment;
import net.minecraft.world.item.enchantment.EnchantmentHelper;
import net.minecraft.world.item.enchantment.Enchantments;
import net.minecraft.server.network.CommonListenerCookie;
import net.minecraft.world.entity.Entity;

/**
 * THROWAWAY (see selftest.sh): a player with a connection, placed like a joining one, for Minecraft 1.21.1.
 * The same steps as vanilla's GameTestHelper.makeMockServerPlayerInLevel.
 */
public final class MockPlayers {

    // The channel of each mock player: it keeps every packet the server sends, in order. A respawn keeps the connection.
    private static final Map<UUID, EmbeddedChannel> CHANNELS = new HashMap<>();

    private MockPlayers() {
    }

    public static ServerPlayer place(MinecraftServer server, ServerLevel level, String name) {
        CommonListenerCookie cookie = CommonListenerCookie.createInitial(new GameProfile(UUID.randomUUID(), name), false);
        ServerPlayer player = new ServerPlayer(server, level, cookie.gameProfile(), cookie.clientInformation());
        Connection connection = new Connection(PacketFlow.SERVERBOUND);
        EmbeddedChannel channel = new EmbeddedChannel(connection);
        CHANNELS.put(player.getUUID(), channel);
        server.getPlayerList().placeNewPlayer(connection, player, cookie);
        return player;
    }

    public static ServerPlayer respawn(MinecraftServer server, ServerPlayer player) {
        return server.getPlayerList().respawn(player, false, Entity.RemovalReason.KILLED);
    }

    /** Every packet sent to this player since the last call, oldest first. */
    public static List<Packet<?>> sent(ServerPlayer player) {
        List<Packet<?>> packets = new ArrayList<>();
        EmbeddedChannel channel = CHANNELS.get(player.getUUID());
        if (channel == null) return packets;
        for (Object message = channel.readOutbound(); message != null; message = channel.readOutbound()) {
            if (message instanceof Packet<?> packet) packets.add(packet);
        }
        return packets;
    }

    // Version helpers: the calls the tests make that changed between 1.20.x and 1.21.1.

    /** The text of a title packet, or null for any other packet. */
    public static Component titleText(Packet<?> packet) {
        return packet instanceof ClientboundSetTitleTextPacket title ? title.text() : null;
    }

    private static Holder<Enchantment> sharpnessHolder(ServerLevel level) {
        return level.registryAccess().registryOrThrow(Registries.ENCHANTMENT).getHolderOrThrow(Enchantments.SHARPNESS);
    }

    public static void enchantSharpness(ServerLevel level, ItemStack stack, int value) {
        stack.enchant(sharpnessHolder(level), value);
    }

    public static int sharpness(ServerLevel level, ItemStack stack) {
        return EnchantmentHelper.getItemEnchantmentLevel(sharpnessHolder(level), stack);
    }

    /** Puts one stack in a bundle the way the Eternal Bundle stores it (the BUNDLE_CONTENTS component). */
    public static void fillBundle(ItemStack bundle, ItemStack content) {
        bundle.set(DataComponents.BUNDLE_CONTENTS, new BundleContents(List.of(content)));
    }

    public static boolean sameData(ItemStack a, ItemStack b) {
        return ItemStack.isSameItemSameComponents(a, b);
    }
}
