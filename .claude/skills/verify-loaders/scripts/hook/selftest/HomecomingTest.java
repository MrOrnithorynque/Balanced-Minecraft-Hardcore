package github.mrornithorynque.bmh.selftest;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;

import github.mrornithorynque.bmh.handlers.AmuletPoints;
import github.mrornithorynque.bmh.handlers.BmhAdvancements;
import github.mrornithorynque.bmh.handlers.HomecomingHandler;
import github.mrornithorynque.bmh.platform.Services;
import github.mrornithorynque.bmh.utilities.BMHGameRules;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.contents.TranslatableContents;
import net.minecraft.network.protocol.Packet;
import net.minecraft.server.level.ServerPlayer;

/**
 * The homecoming (HomecomingHandler). An exile writes a record with the center and the amulet points of that moment. No
 * title before 24 blocks. At 23 blocks the title, the subtitle, the bell come once, and the record is gone.
 * A point made after the exile is no home. A record for another dimension is not read. A new exile replaces the record, a
 * save and a load keep it, and a death with no new exile keeps it. A named point shows its name. The subtitle follows the
 * nights.
 */
final class HomecomingTest implements SelfTest.Test {

    private static final String HOME = HomecomingHandler.TITLE_HOME;

    /** What the client got since the last call. */
    private record Seen(List<String> titles, List<String> subtitles, int sounds, int particles) {
        boolean nothing() {
            return titles.isEmpty() && subtitles.isEmpty() && sounds == 0 && particles == 0;
        }
    }

    @Override
    public String name() {
        return "homecoming";
    }

    @Override
    public void start(SelfTest.Ctx ctx) {
        ctx.rule(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT, true);
        ctx.rule(BMHGameRules.RULE_DISPLAY_TEXT_ON_RESPAWN, true);
        ServerPlayer player = ctx.player("bmh-homecoming", 0, 0);
        BlockPos tavern = ctx.origin.offset(20, 0, 0);
        ((AmuletPoints.Holder) player).bmh$setPoints(List.of(new AmuletPoints.Point("Tavern", tavern)));
        ctx.check("no record before an exile", HomecomingHandler.get(player) == null, "record " + HomecomingHandler.get(player));
        die(ctx, player, back -> firstExile(ctx, back, tavern));
    }

    private static void die(SelfTest.Ctx ctx, ServerPlayer player, Consumer<ServerPlayer> then) {
        MockPlayers.kill(ctx.live(player));
        ctx.later(5, () -> {
            ServerPlayer back = MockPlayers.respawn(ctx.server, ctx.live(player));
            MockPlayers.sent(back);
            then.accept(back);
        });
    }

    private static void firstExile(SelfTest.Ctx ctx, ServerPlayer back, BlockPos tavern) {
        BlockPos spawn = MockPlayers.worldSpawn(ctx.level);
        HomecomingHandler.Record record = HomecomingHandler.get(back);
        if (record == null) {
            ctx.check("the exile wrote a record", false, "record is null");
            return;
        }
        int min = Services.PLATFORM.minDistanceFromBed();
        int max = Services.PLATFORM.maxDistanceFromBed();
        int easyLife = Math.min(9, 1 + ctx.level.getDifficulty().getId());
        int expected = min + (max - min) * easyLife / 9;
        List<HomecomingHandler.Home> expectedHomes = List.of(new HomecomingHandler.Home("", spawn),
                new HomecomingHandler.Home("Tavern", tavern));
        ctx.check("the exile wrote a record with the center and the point", record.homes().equals(expectedHomes),
                "homes " + record.homes() + ", expected " + expectedHomes);
        ctx.check("the record keeps the exile distance", record.distance() == expected,
                "distance " + record.distance() + ", expected " + expected);
        ctx.check("the record keeps the dimension", record.dimension().equals("minecraft:overworld"), record.dimension());

        // A point named after the exile, where the player stands, is no home.
        BlockPos here = back.blockPosition();
        ((AmuletPoints.Holder) back).bmh$setPoints(List.of(new AmuletPoints.Point("Late", here)));
        HomecomingHandler.tick(back);
        ctx.check("a point made after the exile is no home",
                seen(back).nothing() && HomecomingHandler.get(back) != null, "record " + HomecomingHandler.get(back));

        // A record for another dimension is not read, even at the center.
        HomecomingHandler.Record nether = new HomecomingHandler.Record("minecraft:the_nether", record.arrivalTime(),
                record.distance(), record.homes());
        ((HomecomingHandler.Holder) back).bmh$setHomecoming(nether);
        back.setPos(spawn.getX() + 0.5, spawn.getY() + 1, spawn.getZ() + 0.5);
        HomecomingHandler.tick(back);
        ctx.check("a record for another dimension is not read",
                seen(back).nothing() && HomecomingHandler.get(back) == nether, "record " + HomecomingHandler.get(back));
        ((HomecomingHandler.Holder) back).bmh$setHomecoming(record);

        // Just outside the radius: nothing.
        back.setPos(spawn.getX() + 0.5 + HomecomingHandler.RADIUS + 1, spawn.getY() + 1, spawn.getZ() + 0.5);
        HomecomingHandler.tick(back);
        ctx.check("no title at 25 blocks", seen(back).nothing() && HomecomingHandler.get(back) != null, "record " + HomecomingHandler.get(back));

        // Inside: the once-a-second clock of the player tick finds it without a manual call.
        back.setPos(spawn.getX() + 0.5 + HomecomingHandler.RADIUS - 1, spawn.getY() + 1, spawn.getZ() + 0.5);
        ctx.later(25, () -> {
            ServerPlayer live = ctx.live(back);
            Seen seen = seen(live);
            ctx.check("the title comes once at 23 blocks", seen.titles().equals(List.of(HOME)), "titles " + seen.titles());
            ctx.check("the subtitle has no nights", seen.subtitles().equals(List.of(HomecomingHandler.SUBTITLE_NO_NIGHTS + "(1 args)")),
                    "subtitles " + seen.subtitles());
            ctx.check("the bell rings", seen.sounds() >= 1, "sound packets " + seen.sounds());
            ctx.check("no particles are sent", seen.particles() == 0, "particle packets " + seen.particles());
            ctx.check("the record is deleted", HomecomingHandler.get(live) == null, "record " + HomecomingHandler.get(live));
            HomecomingHandler.tick(live);
            ctx.check("nothing comes a second time", seen(live).nothing(), "extra packets");
            secondExile(ctx, live, tavern);
        });
    }

    private static void secondExile(SelfTest.Ctx ctx, ServerPlayer player, BlockPos tavern) {
        die(ctx, player, first -> {
            HomecomingHandler.Record one = HomecomingHandler.get(first);
            ctx.check("a new exile writes a new record", one != null, "record " + one);
            if (one == null) return;
            BlockPos second = ctx.origin.offset(-20, 0, 0);
            ((AmuletPoints.Holder) first).bmh$setPoints(List.of(new AmuletPoints.Point("Second", second)));
            ctx.later(5, () -> die(ctx, first, back -> {
                HomecomingHandler.Record two = HomecomingHandler.get(back);
                ctx.check("a second death replaces the record",
                        two != null && two.arrivalTime() > one.arrivalTime() && two.homes().contains(new HomecomingHandler.Home("Second", second))
                                && !two.homes().contains(new HomecomingHandler.Home("Tavern", tavern)),
                        "first " + one + ", second " + two);
                if (two == null) return;
                // A save and a load keep the record.
                Object data = MockPlayers.saveData(back);
                ((HomecomingHandler.Holder) back).bmh$setHomecoming(null);
                MockPlayers.loadData(back, data);
                ctx.check("a save and a load keep the record", two.equals(HomecomingHandler.get(back)),
                        "loaded " + HomecomingHandler.get(back) + ", saved " + two);
                // A death with no new exile keeps the record.
                ctx.rule(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT, false);
                die(ctx, back, kept -> {
                    ctx.check("a death with no new exile keeps the record", two.equals(HomecomingHandler.get(kept)),
                            "after death " + HomecomingHandler.get(kept) + ", before " + two);
                    ctx.rule(BMHGameRules.RULE_RANDOM_DEATH_SPAWN_POINT, true);
                    namedPoint(ctx, kept, two, second);
                });
            }));
        });
    }

    private static void namedPoint(SelfTest.Ctx ctx, ServerPlayer player, HomecomingHandler.Record record, BlockPos second) {
        player.setPos(second.getX() + 0.5 + 10, second.getY() + 1, second.getZ() + 0.5);
        HomecomingHandler.tick(player);
        Seen seen = seen(player);
        ctx.check("a named point shows its name", seen.titles().equals(List.of("Second")), "titles " + seen.titles());
        ctx.check("the record is deleted at the point", HomecomingHandler.get(player) == null, "record " + HomecomingHandler.get(player));
        nights(ctx, player, record);
    }

    private static void nights(SelfTest.Ctx ctx, ServerPlayer player, HomecomingHandler.Record record) {
        BlockPos spawn = MockPlayers.worldSpawn(ctx.level);
        long now = ctx.level.getGameTime();
        HomecomingHandler.Record threeNights = new HomecomingHandler.Record(record.dimension(), now - 3 * 24000L - 5, 6000,
                List.of(new HomecomingHandler.Home("", spawn)));
        ((HomecomingHandler.Holder) player).bmh$setHomecoming(threeNights);
        player.setPos(spawn.getX() + 0.5, spawn.getY() + 1, spawn.getZ() + 0.5);
        HomecomingHandler.tick(player);
        Seen many = seen(player);
        ctx.check("the subtitle counts nights", many.subtitles().equals(List.of(HomecomingHandler.SUBTITLE + "(2 args)")),
                "subtitles " + many.subtitles());
        ctx.check("the center shows the home title", many.titles().equals(List.of(HOME)), "titles " + many.titles());

        ((HomecomingHandler.Holder) player).bmh$setHomecoming(new HomecomingHandler.Record(record.dimension(), now - 24000L - 5,
                6000, List.of(new HomecomingHandler.Home("", spawn))));
        HomecomingHandler.tick(player);
        ctx.check("the subtitle has a form for one night",
                seen(player).subtitles().equals(List.of(HomecomingHandler.SUBTITLE_ONE_NIGHT + "(1 args)")), "one night");

        // The text follows displayTextOnRespawn. The sound does not.
        ctx.rule(BMHGameRules.RULE_DISPLAY_TEXT_ON_RESPAWN, false);
        ((HomecomingHandler.Holder) player).bmh$setHomecoming(threeNights);
        HomecomingHandler.tick(player);
        Seen quiet = seen(player);
        ctx.check("displayTextOnRespawn off hides the text but not the bell",
                quiet.titles().isEmpty() && quiet.subtitles().isEmpty() && quiet.sounds() >= 1,
                "titles " + quiet.titles() + ", sounds " + quiet.sounds() + ", particles " + quiet.particles());
        awards(ctx, record.dimension());
    }

    // The two advancements of the homecoming. The text rule is still off here: the awards do not depend on it. A near
    // exile gives "Home at last" only, a far one (40000 blocks) gives "The long way home" too.
    private static void awards(SelfTest.Ctx ctx, String dimension) {
        BlockPos spawn = MockPlayers.worldSpawn(ctx.level);
        long now = ctx.level.getGameTime();
        ServerPlayer near = ctx.player("bmh-homecoming-near", 0, 4);
        ServerPlayer far = ctx.player("bmh-homecoming-far", 0, 8);
        ctx.check("a fresh player has neither homecoming advancement",
                !BmhAdvancements.isDone(near, "journey/home_at_last") && !BmhAdvancements.isDone(far, "journey/long_way_home"), "done at the start");
        arriveAt(near, new HomecomingHandler.Record(dimension, now - 5, 5000, List.of(new HomecomingHandler.Home("", spawn))), spawn);
        arriveAt(far, new HomecomingHandler.Record(dimension, now - 5, 40000, List.of(new HomecomingHandler.Home("", spawn))), spawn);
        ctx.check("a near homecoming awards journey/home_at_last", BmhAdvancements.isDone(near, "journey/home_at_last"), "not done");
        ctx.check("a near homecoming (5000 blocks) does not award journey/long_way_home", !BmhAdvancements.isDone(near, "journey/long_way_home"), "done");
        ctx.check("a far homecoming awards journey/home_at_last", BmhAdvancements.isDone(far, "journey/home_at_last"), "not done");
        ctx.check("a far homecoming (40000 blocks) awards journey/long_way_home", BmhAdvancements.isDone(far, "journey/long_way_home"), "not done");
    }

    private static void arriveAt(ServerPlayer player, HomecomingHandler.Record record, BlockPos spawn) {
        ((HomecomingHandler.Holder) player).bmh$setHomecoming(record);
        player.setPos(spawn.getX() + 0.5, spawn.getY() + 1, spawn.getZ() + 0.5);
        HomecomingHandler.tick(player);
    }

    private static String key(Component text) {
        if (text.getContents() instanceof TranslatableContents translatable) {
            int args = translatable.getArgs().length;
            return translatable.getKey() + (args == 0 ? "" : "(" + args + " args)");
        }
        return text.getString();
    }

    private static Seen seen(ServerPlayer player) {
        List<String> titles = new ArrayList<>();
        List<String> subtitles = new ArrayList<>();
        int sounds = 0;
        int particles = 0;
        for (Packet<?> packet : MockPlayers.sent(player)) {
            Component title = MockPlayers.titleText(packet);
            Component subtitle = MockPlayers.subtitleText(packet);
            String type = packet.getClass().getSimpleName();
            if (title != null) titles.add(key(title));
            else if (subtitle != null) subtitles.add(key(subtitle));
            else if (type.contains("Sound")) sounds++;
            else if (type.contains("Particles")) particles++;
        }
        return new Seen(titles, subtitles, sounds, particles);
    }
}
