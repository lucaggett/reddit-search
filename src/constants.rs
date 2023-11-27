use std::collections::HashMap;
// these values were pre-calculated to make the progress bar more accurate.
// precomputed values only exist for the reddit dataset as linked in the repo/help text.
pub(crate) fn create_line_count_map() -> HashMap<&'static str, u64> {
    vec![
        ("RC_2005-12.zst", 1075),
        ("RC_2006-01.zst", 3666),
        ("RC_2006-02.zst", 9095),
        ("RC_2006-03.zst", 13859),
        ("RC_2006-04.zst", 19090),
        ("RC_2006-05.zst", 26859),
        ("RC_2006-06.zst", 29163),
        ("RC_2006-07.zst", 37031),
        ("RC_2006-08.zst", 50559),
        ("RC_2006-09.zst", 50675),
        ("RC_2006-10.zst", 54148),
        ("RC_2006-11.zst", 62021),
        ("RC_2006-12.zst", 61018),
        ("RC_2007-01.zst", 81341),
        ("RC_2007-02.zst", 95634),
        ("RC_2007-03.zst", 112444),
        ("RC_2007-04.zst", 126773),
        ("RC_2007-05.zst", 170097),
        ("RC_2007-06.zst", 178800),
        ("RC_2007-07.zst", 203319),
        ("RC_2007-08.zst", 225111),
        ("RC_2007-09.zst", 259497),
        ("RC_2007-10.zst", 274170),
        ("RC_2007-11.zst", 372983),
        ("RC_2007-12.zst", 363390),
        ("RC_2008-01.zst", 452990),
        ("RC_2008-02.zst", 441768),
        ("RC_2008-03.zst", 463728),
        ("RC_2008-04.zst", 468317),
        ("RC_2008-05.zst", 536380),
        ("RC_2008-06.zst", 577684),
        ("RC_2008-07.zst", 592610),
        ("RC_2008-08.zst", 595959),
        ("RC_2008-09.zst", 680892),
        ("RC_2008-10.zst", 789874),
        ("RC_2008-11.zst", 792310),
        ("RC_2008-12.zst", 850359),
        ("RC_2009-01.zst", 1051649),
        ("RC_2009-02.zst", 944711),
        ("RC_2009-03.zst", 1048643),
        ("RC_2009-04.zst", 1094599),
        ("RC_2009-05.zst", 1201257),
        ("RC_2009-06.zst", 1258750),
        ("RC_2009-07.zst", 1470290),
        ("RC_2009-08.zst", 1750688),
        ("RC_2009-09.zst", 2032276),
        ("RC_2009-10.zst", 2242017),
        ("RC_2009-11.zst", 2207444),
        ("RC_2009-12.zst", 2560510),
        ("RC_2010-01.zst", 2884096),
        ("RC_2010-02.zst", 2687779),
        ("RC_2010-03.zst", 3228254),
        ("RC_2010-04.zst", 3209898),
        ("RC_2010-05.zst", 3267363),
        ("RC_2010-06.zst", 3532867),
        ("RC_2010-07.zst", 4032737),
        ("RC_2010-08.zst", 4247982),
        ("RC_2010-09.zst", 4704069),
        ("RC_2010-10.zst", 5032368),
        ("RC_2010-11.zst", 5689002),
        ("RC_2010-12.zst", 5972642),
        ("RC_2011-01.zst", 6603329),
        ("RC_2011-02.zst", 6363114),
        ("RC_2011-03.zst", 7556165),
        ("RC_2011-04.zst", 7571398),
        ("RC_2011-05.zst", 8803949),
        ("RC_2011-06.zst", 9766511),
        ("RC_2011-07.zst", 10557466),
        ("RC_2011-08.zst", 12316144),
        ("RC_2011-09.zst", 12150412),
        ("RC_2011-10.zst", 13470278),
        ("RC_2011-11.zst", 13621533),
        ("RC_2011-12.zst", 14509469),
        ("RC_2012-01.zst", 16350205),
        ("RC_2012-02.zst", 16015695),
        ("RC_2012-03.zst", 17881943),
        ("RC_2012-04.zst", 19044534),
        ("RC_2012-05.zst", 20388260),
        ("RC_2012-06.zst", 21897913),
        ("RC_2012-07.zst", 24087517),
        ("RC_2012-08.zst", 25703326),
        ("RC_2012-09.zst", 23419524),
        ("RC_2012-10.zst", 24788236),
        ("RC_2012-11.zst", 24648302),
        ("RC_2012-12.zst", 26080276),
        ("RC_2013-01.zst", 30365867),
        ("RC_2013-02.zst", 27213960),
        ("RC_2013-03.zst", 30771274),
        ("RC_2013-04.zst", 33259557),
        ("RC_2013-05.zst", 33126225),
        ("RC_2013-06.zst", 32648247),
        ("RC_2013-07.zst", 34922133),
        ("RC_2013-08.zst", 34766579),
        ("RC_2013-09.zst", 31990369),
        ("RC_2013-10.zst", 35940040),
        ("RC_2013-11.zst", 37396497),
        ("RC_2013-12.zst", 39810216),
        ("RC_2014-01.zst", 42420655),
        ("RC_2014-02.zst", 38703362),
        ("RC_2014-03.zst", 42459956),
        ("RC_2014-04.zst", 42440735),
        ("RC_2014-05.zst", 42514094),
        ("RC_2014-06.zst", 41990650),
        ("RC_2014-07.zst", 46868899),
        ("RC_2014-08.zst", 46990813),
        ("RC_2014-09.zst", 44992201),
        ("RC_2014-10.zst", 47497520),
        ("RC_2014-11.zst", 46118074),
        ("RC_2014-12.zst", 48807699),
        ("RC_2015-01.zst", 53851542),
        ("RC_2015-02.zst", 48342747),
        ("RC_2015-03.zst", 54564441),
        ("RC_2015-04.zst", 55005780),
        ("RC_2015-05.zst", 54504410),
        ("RC_2015-06.zst", 54258492),
        ("RC_2015-07.zst", 58451788),
        ("RC_2015-08.zst", 58075327),
        ("RC_2015-09.zst", 55574825),
        ("RC_2015-10.zst", 59494045),
        ("RC_2015-11.zst", 57117500),
        ("RC_2015-12.zst", 58523312),
        ("RC_2016-01.zst", 61991732),
        ("RC_2016-02.zst", 59189875),
        ("RC_2016-03.zst", 63918864),
        ("RC_2016-04.zst", 64271256),
        ("RC_2016-05.zst", 65212004),
        ("RC_2016-06.zst", 65867743),
        ("RC_2016-07.zst", 66974735),
        ("RC_2016-08.zst", 69654819),
        ("RC_2016-09.zst", 67024973),
        ("RC_2016-10.zst", 71826553),
        ("RC_2016-11.zst", 71022319),
        ("RC_2016-12.zst", 72942967),
        ("RC_2017-01.zst", 78946585),
        ("RC_2017-02.zst", 70609487),
        ("RC_2017-03.zst", 79723106),
        ("RC_2017-04.zst", 77478009),
        ("RC_2017-05.zst", 79810360),
        ("RC_2017-06.zst", 79901711),
        ("RC_2017-07.zst", 81798725),
        ("RC_2017-08.zst", 84658503),
        ("RC_2017-09.zst", 83165192),
        ("RC_2017-10.zst", 85828912),
        ("RC_2017-11.zst", 84965681),
        ("RC_2017-12.zst", 85973810),
        ("RC_2018-01.zst", 91558594),
        ("RC_2018-02.zst", 86467179),
        ("RC_2018-03.zst", 96490262),
        ("RC_2018-04.zst", 98101232),
        ("RC_2018-05.zst", 100109100),
        ("RC_2018-06.zst", 100009462),
        ("RC_2018-07.zst", 108151359),
        ("RC_2018-08.zst", 107330940),
        ("RC_2018-09.zst", 104473929),
        ("RC_2018-10.zst", 112346556),
        ("RC_2018-11.zst", 112573001),
        ("RC_2018-12.zst", 121953600),
        ("RC_2019-01.zst", 129386587),
        ("RC_2019-02.zst", 120645639),
        ("RC_2019-03.zst", 137650471),
        ("RC_2019-04.zst", 138473643),
        ("RC_2019-05.zst", 142463421),
        ("RC_2019-06.zst", 134172939),
        ("RC_2019-07.zst", 145965083),
        ("RC_2019-08.zst", 146854393),
        ("RC_2019-09.zst", 137540219),
        ("RC_2019-10.zst", 145909884),
        ("RC_2019-11.zst", 138512489),
        ("RC_2019-12.zst", 146012313),
        ("RC_2020-01.zst", 153498208),
        ("RC_2020-02.zst", 148386817),
        ("RC_2020-03.zst", 166266315),
        ("RC_2020-04.zst", 178511581),
        ("RC_2020-05.zst", 189993779),
        ("RC_2020-06.zst", 187914434),
        ("RC_2020-07.zst", 194244994),
        ("RC_2020-08.zst", 196099301),
        ("RC_2020-09.zst", 182549761),
        ("RC_2020-10.zst", 186583890),
        ("RC_2020-11.zst", 186083723),
        ("RC_2020-12.zst", 191317162),
        ("RC_2021-01.zst", 210496207),
        ("RC_2021-02.zst", 193510365),
        ("RC_2021-03.zst", 207454415),
        ("RC_2021-04.zst", 204573086),
        ("RC_2021-05.zst", 217655366),
        ("RC_2021-06.zst", 208027069),
        ("RC_2021-07.zst", 210955954),
        ("RC_2021-08.zst", 225681244),
        ("RC_2021-09.zst", 220086513),
        ("RC_2021-10.zst", 227527379),
        ("RC_2021-11.zst", 228289963),
        ("RC_2021-12.zst", 235807471),
        ("RC_2022-01.zst", 256766679),
        ("RC_2022-02.zst", 219927645),
        ("RC_2022-03.zst", 236554668),
        ("RC_2022-04.zst", 231188077),
        ("RC_2022-05.zst", 230492108),
        ("RC_2022-06.zst", 218842949),
        ("RC_2022-07.zst", 242504279),
        ("RC_2022-08.zst", 247215325),
        ("RC_2022-09.zst", 234131223),
        ("RC_2022-10.zst", 237365072),
        ("RC_2022-11.zst", 229478878),
        ("RC_2022-12.zst", 238862690),
    ].into_iter().collect()
}

// the presets are a hashmap of strings (preset names) mapping to vectors of values (the field strings to include)
pub(crate) fn get_presets() -> HashMap<&'static str, Vec<&'static str>> {
    HashMap::from([
        ("en_news", vec!["subreddit:news", "subreddit:worldnews"]),
        ("en_politics", vec!["subreddit:politics", "subreddit:PoliticalDiscussion", "subreddit:geopolitics", "subreddit:NeutralPolitics", "subreddit:Ask_Politics", "subreddit:PoliticalHumor", "subreddit:PoliticalCompassMemes", "subreddit:PoliticalMemes", "subreddit:PoliticalDiscussion", "subreddit:ShitPoliticsSay"]),
        ("en_science", vec!["subreddit:science", "subreddit:EverythingScience", "subreddit:AskScience", "subreddit:EverythingScience"]),
        ("en_hate_speech", vec!["subreddit:Physical_Removal", "subreddit:MillionDollarExtreme", "subreddit:GasTheKikes", "subreddit:FatPeopleHate", "subreddit:Beatingwomen", "subreddit:niggers", "subreddit:UncensoredNews"]),
        ("controversial", vec!["controversiality:1"]),
    ])
}